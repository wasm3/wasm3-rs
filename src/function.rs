use core::cmp::{Eq, PartialEq};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::slice;
use core::str;

use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::utils::cstr_to_str;
use crate::wasm3_priv;
use crate::{WasmArgs, WasmType};

/// Calling Context for a host function.
pub struct CallContext {
    runtime: NonNull<ffi::M3Runtime>,
    mem: NonNull<ffi::M3Memory>,
}

impl CallContext {
    pub(crate) fn from_rt_mem(
        runtime: NonNull<ffi::M3Runtime>,
        mem: NonNull<ffi::M3Memory>,
    ) -> CallContext {
        CallContext { runtime, mem }
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub unsafe fn memory(&self) -> *const [u8] {
        let mallocated = (*self.mem.as_ptr()).mallocated;
        let len = (*mallocated).length as usize;
        let data = if len == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            mallocated.offset(1).cast()
        };
        ptr::slice_from_raw_parts(data, len)
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub unsafe fn memory_mut(&self) -> *mut [u8] {
        let mallocated = (*self.mem.as_ptr()).mallocated;
        let len = (*mallocated).length as usize;
        let data = if len == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            mallocated.offset(1).cast()
        };
        ptr::slice_from_raw_parts_mut(data, len)
    }
}

// redefine of ffi::RawCall without the Option<T> around it
/// Type of a raw host function for wasm3.
pub type RawCall = unsafe extern "C" fn(
    runtime: ffi::IM3Runtime,
    _sp: ffi::m3stack_t,
    _mem: *mut cty::c_void,
) -> *const cty::c_void;

pub(crate) type NNM3Function = NonNull<ffi::M3Function>;

/// A callable wasm3 function.
/// This has a generic `call` function for up to 26 parameters emulating an overloading behaviour without having to resort to tuples.
/// These are hidden to not pollute the documentation.
#[derive(Debug, Copy, Clone)]
pub struct Function<'rt, ARGS, RET> {
    raw: NNM3Function,
    rt: &'rt Runtime,
    _pd: PhantomData<*const (ARGS, RET)>,
}

impl<'rt, ARGS, RET> Eq for Function<'rt, ARGS, RET> {}
impl<'rt, ARGS, RET> PartialEq for Function<'rt, ARGS, RET> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<'rt, ARGS, RET> Function<'rt, ARGS, RET>
where
    ARGS: WasmArgs,
    RET: WasmType,
{
    /// The name of the import module of this function.
    pub fn import_module_name(&self) -> &str {
        unsafe { cstr_to_str(self.raw.as_ref().import.moduleUtf8) }
    }

    /// The name of this function.
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str(self.raw.as_ref().name) }
    }
}

impl<'rt, ARGS, RET> Function<'rt, ARGS, RET>
where
    ARGS: WasmArgs,
    RET: WasmType,
{
    pub(crate) fn validate_sig(mut func: NNM3Function) -> Result<()> {
        let &ffi::M3FuncType {
            returnType: ret,
            argTypes: ref args,
            numArgs: num,
            ..
        } = unsafe { &*func.as_mut().funcType };
        // argTypes is actually dynamically sized.
        let args = unsafe { slice::from_raw_parts(args.as_ptr(), num as usize) };
        match RET::TYPE_INDEX == ret && ARGS::validate_types(args) {
            true => Ok(()),
            false => Err(Error::InvalidFunctionSignature),
        }
    }

    #[inline]
    pub(crate) fn from_raw(rt: &'rt Runtime, raw: NNM3Function) -> Result<Self> {
        Self::validate_sig(raw)?;
        let this = Function {
            raw,
            rt,
            _pd: PhantomData,
        };
        // make sure the function is compiled
        this.compile()
    }

    #[inline]
    pub(crate) fn compile(self) -> Result<Self> {
        unsafe {
            if self.raw.as_ref().compiled.is_null() {
                Error::from_ffi_res(wasm3_priv::Compile_Function(self.raw.as_ptr()))?;
            }
        };
        Ok(self)
    }

    fn call_impl(&self, args: ARGS) -> Result<RET> {
        let stack = self.rt.stack_mut();
        let ret = unsafe {
            args.push_on_stack(stack);
            Self::call_impl_(
                self.raw.as_ref().compiled,
                stack.cast(),
                self.rt.mallocated(),
                666,
                core::f64::NAN,
            )
        };
        Error::from_ffi_res(ret.cast()).map(|()| unsafe { RET::pop_from_stack(stack.cast()) })
    }

    #[inline]
    unsafe fn call_impl_(
        _pc: ffi::pc_t,
        _sp: ffi::m3stack_t,
        _mem: *mut ffi::M3MemoryHeader,
        _r0: ffi::m3reg_t,
        _fp0: f64,
    ) -> ffi::m3ret_t {
        let possible_trap = ffi::m3_Yield();
        if !possible_trap.is_null() {
            possible_trap.cast()
        } else {
            (*_pc.cast::<ffi::IM3Operation>()).expect("IM3Operation was null")(
                _pc.add(1),
                _sp,
                _mem,
                _r0,
                _fp0,
            )
        }
    }
}

macro_rules! func_call_impl {
    ($($types:ident),*) => { func_call_impl!(@rec [$($types,)*] []); };
    (@rec [] [$($types:ident,)*]) => { func_call_impl!(@do_impl $($types,)*); };
    (@rec [$head:ident, $($tail:ident,)*] [$($types:ident,)*]) => {
        func_call_impl!(@do_impl $($types,)*);
        func_call_impl!(@rec [$($tail,)*] [$($types,)* $head,]);
    };
    (@do_impl) => {};
    (@do_impl $($types:ident,)*) => {
    #[doc(hidden)] // this really pollutes the documentation
        impl<'rt, $($types,)* RET> Function<'rt, ($($types,)*), RET>
        where
            RET: WasmType,
            ($($types,)*): WasmArgs,
        {
            #[inline]
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub fn call(&self, $($types: $types),*) -> Result<RET> {
                self.call_impl(($($types,)*))
            }
        }
    };
}
func_call_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

impl<'rt, ARG, RET> Function<'rt, ARG, RET>
where
    RET: WasmType,
    ARG: crate::WasmArg,
{
    /// Calls this function with the given parameter.
    /// This is implemented with variable arguments depending on the functions ARGS type.
    #[inline]
    pub fn call(&self, arg: ARG) -> Result<RET> {
        self.call_impl(arg)
    }
}

impl<'rt, RET> Function<'rt, (), RET>
where
    RET: WasmType,
{
    /// Calls this function.
    /// This is implemented with variable arguments depending on the functions ARGS type.
    #[inline]
    pub fn call(&self) -> Result<RET> {
        self.call_impl(())
    }
}
