use core::marker::PhantomData;
use core::ptr::NonNull;
use core::str;

use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::utils::cstr_to_str;
use crate::wasm3_priv;
use crate::{WasmArgs, WasmType};

// redefine of ffi::RawCall without the Option<T> around it
pub type RawCall = unsafe extern "C" fn(
    runtime: ffi::IM3Runtime,
    _sp: *mut u64,
    _mem: *mut cty::c_void,
) -> *const cty::c_void;

pub(crate) type NNM3Function = NonNull<ffi::M3Function>;

/// A callable wasm3 function.
#[derive(Debug)]
pub struct Function<'env, 'rt, ARGS, RET> {
    raw: NNM3Function,
    rt: &'rt Runtime<'env>,
    _pd: PhantomData<(ARGS, RET)>,
}

impl<'env, 'rt, ARGS, RET> Function<'env, 'rt, ARGS, RET>
where
    ARGS: WasmArgs,
    RET: WasmType,
{
    pub(crate) fn validate_sig(mut func: NNM3Function) -> bool {
        let &ffi::M3FuncType {
            returnType: ret,
            argTypes: ref args,
            numArgs: num,
            ..
        } = unsafe { &*func.as_mut().funcType };
        RET::TYPE_INDEX == ret && ARGS::validate_types(&args[..num as usize])
    }

    #[inline]
    pub(crate) fn from_raw(rt: &'rt Runtime<'env>, raw: NNM3Function) -> Result<Self> {
        if Self::validate_sig(raw) {
            let this = Function {
                raw,
                rt,
                _pd: PhantomData,
            };
            this.compile()
        } else {
            Err(Error::InvalidFunctionSignature)
        }
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

    /// The name of the import module of this function.
    pub fn import_module_name(&self) -> &str {
        unsafe { cstr_to_str(self.raw.as_ref().import.moduleUtf8) }
    }

    /// The name of this function.
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str(self.raw.as_ref().name) }
    }

    fn call_impl(&self, args: ARGS) -> Result<RET> {
        let stack = unsafe { self.rt.stack_mut() };
        args.put_on_stack(stack);
        let ret = unsafe {
            Self::call_impl_(
                self.raw.as_ref().compiled,
                stack.as_mut_ptr(),
                self.rt.mallocated(),
                666,
                core::f64::NAN,
            )
        };
        match self.rt.rt_error() {
            Err(e) if ret.is_null() => Err(e),
            _ => Ok(RET::from_u64(stack[0])),
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
    (@do_impl $($types:ident,)*) => {
        #[doc(hidden)] // this really pollutes the documentation
        impl<'env, 'rt, $($types,)* RET> Function<'env, 'rt, ($($types,)*), RET>
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

impl<'env, 'rt, ARG, RET> Function<'env, 'rt, ARG, RET>
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
