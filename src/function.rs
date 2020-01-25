use std::marker::PhantomData;

use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::{WasmArgs, WasmType};

pub struct Function<'env, 'rt, ARGS, RET> {
    raw: ffi::IM3Function,
    rt: &'rt Runtime<'env>,
    _pd: PhantomData<(ARGS, RET)>,
}

impl<'env, 'rt, ARGS, RET> Function<'env, 'rt, ARGS, RET>
where
    ARGS: WasmArgs,
    RET: WasmType,
{
    #[inline]
    pub(crate) fn from_raw(rt: &'rt Runtime<'env>, raw: ffi::IM3Function) -> Result<Self> {
        let &ffi::M3FuncType {
            returnType: ret,
            argTypes: args,
            numArgs: num,
            ..
        } = unsafe { &*(*raw).funcType };
        if RET::TYPE_INDEX == ret && ARGS::validate_types(&args[..num as usize]) {
            Ok(Function {
                raw,
                rt,
                _pd: PhantomData,
            })
        } else {
            Err(Error::InvalidFunctionSignature)
        }
    }

    pub fn import_module_name(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(
                std::ffi::CStr::from_ptr((*self.raw).import.moduleUtf8).to_bytes(),
            )
        }
    }

    pub fn name(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::ffi::CStr::from_ptr((*self.raw).name).to_bytes())
        }
    }

    fn call_impl(&self, args: ARGS) -> Result<RET> {
        let stack = self.rt.stack_mut();
        args.put_on_stack(stack);
        let ret = unsafe {
            Self::call_impl_(
                (*self.raw).compiled,
                stack.as_mut_ptr(),
                self.rt.mallocated(),
                666,
                std::f64::NAN,
            )
        };
        if !self.rt.has_errored() && ret.is_null() {
            Ok(RET::fetch_from_stack(stack))
        } else {
            Err(Error::Wasm3("FIXME: error case"))
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
        ffi::m3Yield();
        (*_pc.cast::<ffi::IM3Operation>()).unwrap()(_pc.add(1), _sp, _mem, _r0, _fp0)
    }
}

macro_rules! func_call_impl {
    ($($types:ident),*) => {
        func_call_impl!(@rec __DUMMY__T, $($types),*);
    };
    (@rec $types:ident) => {};
    (@rec $_:ident, $($types:ident),*) => {
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
        func_call_impl!(@rec $($types),*);
    };
}
func_call_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

impl<'env, 'rt, T, RET> Function<'env, 'rt, T, RET>
where
    RET: WasmType,
    T: crate::WasmArg,
{
    #[inline]
    pub fn call(&self, t: T) -> Result<RET> {
        self.call_impl(t)
    }
}
