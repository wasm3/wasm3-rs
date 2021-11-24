use core::cmp::{Eq, PartialEq};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::str;

use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::utils::cstr_to_str;
use crate::{Module, WasmArg, WasmArgs, WasmType};

/// Calling Context for a host function.
pub struct CallContext<'cc> {
    runtime: NonNull<ffi::M3Runtime>,
    _pd: PhantomData<fn(&'cc ()) -> &'cc ()>,
}

impl<'cc> CallContext<'cc> {
    pub(crate) fn from_rt(runtime: NonNull<ffi::M3Runtime>) -> CallContext<'cc> {
        CallContext {
            runtime,
            _pd: PhantomData,
        }
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub fn memory(&self) -> *const [u8] {
        let mut memory_size = 0u32;
        let data = unsafe { ffi::m3_GetMemory(self.runtime.as_ptr(), &mut memory_size, 0) };
        ptr::slice_from_raw_parts(data, memory_size as usize)
    }

    /// Returns the raw memory of the runtime associated with this context.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub fn memory_mut(&self) -> *mut [u8] {
        let mut memory_size = 0u32;
        let data = unsafe { ffi::m3_GetMemory(self.runtime.as_ptr(), &mut memory_size, 0) };
        ptr::slice_from_raw_parts_mut(data, memory_size as usize)
    }
}

// redefine of ffi::RawCall without the Option<T> around it
/// Type of a raw host function for wasm3.
pub type RawCall = unsafe extern "C" fn(
    runtime: ffi::IM3Runtime,
    ctx: ffi::IM3ImportContext,
    _sp: *mut u64,
    _mem: *mut cty::c_void,
) -> *const cty::c_void;

pub(crate) type NNM3Function = NonNull<ffi::M3Function>;

/// A callable wasm3 function.
/// This has a generic `call` function for up to 26 parameters emulating an overloading behaviour without having to resort to tuples.
/// These are hidden to not pollute the documentation.
#[derive(Debug, Copy, Clone)]
pub struct Function<'rt, Args, Ret> {
    raw: NNM3Function,
    rt: &'rt Runtime,
    _pd: PhantomData<*const (Args, Ret)>,
}

impl<'rt, Args, Ret> Eq for Function<'rt, Args, Ret> {}
impl<'rt, Args, Ret> PartialEq for Function<'rt, Args, Ret> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<'rt, Args, Ret> Hash for Function<'rt, Args, Ret> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<'rt, Args, Ret> Function<'rt, Args, Ret>
where
    Args: WasmArgs,
    Ret: WasmType,
{
    /// The name of this function.
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str(ffi::m3_GetFunctionName(self.raw.as_ptr())) }
    }

    /// The module containing this function.
    pub fn module(&self) -> Option<Module<'rt>> {
        let module = unsafe { ffi::m3_GetFunctionModule(self.raw.as_ptr()) };
        if !module.is_null() {
            Some(Module::from_raw(self.rt, module))
        } else {
            None
        }
    }
}

impl<'rt, Args, Ret> Function<'rt, Args, Ret>
where
    Args: WasmArgs,
    Ret: WasmType,
{
    fn validate_sig(func: NNM3Function) -> bool {
        let num_args = unsafe { ffi::m3_GetArgCount(func.as_ptr()) };
        let args = (0..num_args).map(|i| unsafe { ffi::m3_GetArgType(func.as_ptr(), i) });
        if !Args::validate_types(args) {
            return false;
        }

        let num_rets = unsafe { ffi::m3_GetRetCount(func.as_ptr()) };
        match num_rets {
            0 => Ret::TYPE_INDEX == ffi::M3ValueType::c_m3Type_none,
            1 => {
                let ret = unsafe { ffi::m3_GetRetType(func.as_ptr(), 0) };
                Ret::TYPE_INDEX == ret
            }
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn from_raw(rt: &'rt Runtime, raw: NNM3Function) -> Result<Self> {
        if !Self::validate_sig(raw) {
            return Err(Error::InvalidFunctionSignature);
        }
        Ok(Function {
            raw,
            rt,
            _pd: PhantomData,
        })
    }

    fn get_call_result(&self) -> Result<Ret> {
        unsafe {
            let mut ret = core::mem::MaybeUninit::<Ret>::uninit();
            let result = ffi::m3_GetResultsV(self.raw.as_ptr(), ret.as_mut_ptr());
            Error::from_ffi_res(result)?;
            Ok(ret.assume_init())
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
        impl<'rt, $($types,)* Ret> Function<'rt, ($($types,)*), Ret>
        where
            Ret: WasmType,
            ($($types,)*): WasmArgs,
        {
            #[inline]
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub fn call(&self, $($types: $types),*) -> Result<Ret> {
                let result = unsafe { ffi::m3_CallV(self.raw.as_ptr(), $($types,)*) };
                Error::from_ffi_res(result)?;
                self.get_call_result()
            }
        }
    };
}
func_call_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

impl<'rt, ARG, Ret> Function<'rt, ARG, Ret>
where
    Ret: WasmType,
    ARG: WasmArg,
{
    /// Calls this function with the given parameter.
    /// This is implemented with variable arguments depending on the functions Args type.
    #[inline]
    pub fn call(&self, arg: ARG) -> Result<Ret> {
        let result = unsafe { ffi::m3_CallV(self.raw.as_ptr(), arg) };
        Error::from_ffi_res(result)?;
        self.get_call_result()
    }
}

impl<'rt, Ret> Function<'rt, (), Ret>
where
    Ret: WasmType,
{
    /// Calls this function.
    /// This is implemented with variable arguments depending on the functions Args type.
    #[inline]
    pub fn call(&self) -> Result<Ret> {
        let result = unsafe { ffi::m3_CallV(self.raw.as_ptr()) };
        Error::from_ffi_res(result)?;
        self.get_call_result()
    }
}
