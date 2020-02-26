use core::cmp::{Eq, PartialEq};
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::str;

use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::utils::{cstr_to_str, rt_check};
use crate::wasm3_priv;
use crate::{WasmArgs, WasmType};

// redefine of ffi::RawCall without the Option<T> around it
pub type RawCall = unsafe extern "C" fn(
    runtime: ffi::IM3Runtime,
    _sp: *mut u64,
    _mem: *mut cty::c_void,
) -> *const cty::c_void;

pub(crate) type NNM3Function = NonNull<ffi::M3Function>;

/// A wasm3 function token which can be used to call the corresponding function in the runtime.
/// This has a generic `call` function for up to 26 parameters.
/// These are hidden to not pollute the documentation.
/// This is just a token which can be used to perform the desired actions on the runtime it belongs to.
#[derive(Debug, Copy, Clone)]
pub struct Function<ARGS, RET> {
    raw: NNM3Function,
    raw_rt: ffi::IM3Runtime,
    _pd: PhantomData<(ARGS, RET)>,
}

impl<ARGS, RET> Eq for Function<ARGS, RET> {}
impl<ARGS, RET> PartialEq for Function<ARGS, RET> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<ARGS, RET> Function<ARGS, RET>
where
    ARGS: WasmArgs,
    RET: WasmType,
{
    /// The name of the import module of this function.
    pub fn import_module_name<'rt>(&self, rt: &'rt Runtime) -> &'rt str {
        rt_check(rt, self.raw_rt);
        unsafe { cstr_to_str(self.raw.as_ref().import.moduleUtf8) }
    }

    /// The name of this function.
    pub fn name<'rt>(&self, rt: &'rt Runtime) -> &'rt str {
        rt_check(rt, self.raw_rt);
        unsafe { cstr_to_str(self.raw.as_ref().name) }
    }
}

impl<ARGS, RET> Function<ARGS, RET>
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
        match RET::TYPE_INDEX == ret && ARGS::validate_types(&args[..num as usize]) {
            true => Ok(()),
            false => Err(Error::InvalidFunctionSignature),
        }
    }

    #[inline]
    pub(crate) fn from_raw(raw_rt: ffi::IM3Runtime, raw: NNM3Function) -> Result<Self> {
        Self::validate_sig(raw)?;
        let this = Function {
            raw,
            raw_rt,
            _pd: PhantomData,
        };
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

    fn call_impl(&self, rt: &mut Runtime, args: ARGS) -> Result<RET> {
        rt_check(rt, self.raw_rt);
        let (ret, slot) = unsafe {
            let _mem = rt.mallocated();
            let stack = rt.stack_mut();
            args.put_on_stack(stack);
            let ret = Self::call_impl_(
                self.raw.as_ref().compiled,
                stack.as_mut_ptr(),
                _mem,
                666,
                core::f64::NAN,
            );
            let &slot = stack.get_unchecked(0);
            (ret, slot)
        };
        match rt.rt_error() {
            Err(e) if ret.is_null() => Err(e),
            _ => Ok(RET::from_u64(slot)),
        }
    }

    #[inline]
    unsafe fn call_impl_(
        _pc: ffi::pc_t,
        _sp: *mut u64,
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
        impl<$($types,)* RET> Function<($($types,)*), RET>
        where
            RET: WasmType,
            ($($types,)*): WasmArgs,
        {
            #[inline]
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub fn call(&self, rt: &mut Runtime, $($types: $types),*) -> Result<RET> {
                self.call_impl(rt, ($($types,)*))
            }
        }
    };
}
func_call_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

impl<ARG, RET> Function<ARG, RET>
where
    RET: WasmType,
    ARG: crate::WasmArg,
{
    /// Calls this function with the given parameter.
    /// This is implemented with variable arguments depending on the functions ARGS type.
    #[inline]
    pub fn call(&self, rt: &mut Runtime, arg: ARG) -> Result<RET> {
        self.call_impl(rt, arg)
    }
}

impl<RET> Function<(), RET>
where
    RET: WasmType,
{
    /// Calls this function.
    /// This is implemented with variable arguments depending on the functions ARGS type.
    #[inline]
    pub fn call(&self, rt: &mut Runtime) -> Result<RET> {
        self.call_impl(rt, ())
    }
}
