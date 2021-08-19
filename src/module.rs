use alloc::boxed::Box;
use core::ptr::{self, NonNull};
use core::slice;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::{CallContext, Function, NNM3Function, RawCall};
use crate::runtime::Runtime;
use crate::utils::{cstr_to_str, eq_cstr_str};
use crate::wasm3_priv;

/// A parsed module which can be loaded into a [`Runtime`].
pub struct ParsedModule {
    raw: ffi::IM3Module,
    env: Environment,
}

impl ParsedModule {
    /// Parses a wasm module from raw bytes.
    pub fn parse(env: &Environment, bytes: &[u8]) -> Result<Self> {
        assert!(bytes.len() <= !0u32 as usize);
        let mut module = ptr::null_mut();
        let res = unsafe {
            ffi::m3_ParseModule(
                env.as_ptr(),
                &mut module,
                bytes.as_ptr(),
                bytes.len() as u32,
            )
        };
        Error::from_ffi_res(res).map(|_| ParsedModule {
            raw: module,
            env: env.clone(),
        })
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Module {
        self.raw
    }

    /// The environment this module was parsed in.
    pub fn environment(&self) -> &Environment {
        &self.env
    }
}

impl Drop for ParsedModule {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeModule(self.raw) };
    }
}

/// A loaded module belonging to a specific runtime. Allows for linking and looking up functions.
// needs no drop as loaded modules will be cleaned up by the runtime
pub struct Module<'rt> {
    raw: ffi::IM3Module,
    rt: &'rt Runtime,
}

impl<'rt> Module<'rt> {
    /// Parses a wasm module from raw bytes.
    #[inline]
    pub fn parse(environment: &Environment, bytes: &[u8]) -> Result<ParsedModule> {
        ParsedModule::parse(environment, bytes)
    }

    /// Links the given function to the corresponding module and function name.
    /// This allows linking a more verbose function, as it gets access to the unsafe
    /// runtime parts. For easier use the [`make_func_wrapper`] should be used to create
    /// the unsafe facade for your function that then can be passed to this.
    ///
    /// For a simple API see [`link_closure`] which takes a closure instead.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    ///
    /// [`link_closure`]: #method.link_closure
    pub fn link_function<Args, Ret>(
        &mut self,
        module_name: &str,
        function_name: &str,
        f: RawCall,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let func = self.find_import_function(module_name, function_name)?;
        Function::<'_, Args, Ret>::validate_sig(func)
            .and_then(|_| unsafe { self.link_func_impl(func, f) })
    }

    /// Links the given closure to the corresponding module and function name.
    /// This boxes the closure and therefor requires a heap allocation.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn link_closure<Args, Ret, F>(
        &mut self,
        module_name: &str,
        function_name: &str,
        closure: F,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
        F: for<'cc> FnMut(CallContext<'cc>, Args) -> Ret + 'static,
    {
        let func = self.find_import_function(module_name, function_name)?;
        Function::<'_, Args, Ret>::validate_sig(func)?;
        let mut closure = Box::pin(closure);
        unsafe { self.link_closure_impl(func, closure.as_mut().get_unchecked_mut()) }?;
        self.rt.push_closure(closure);
        Ok(())
    }

    /// Looks up a function by the given name in this module.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn find_function<Args, Ret>(&self, function_name: &str) -> Result<Function<'rt, Args, Ret>>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let func = unsafe {
            slice::from_raw_parts_mut(
                if (*self.raw).functions.is_null() {
                    NonNull::dangling().as_ptr()
                } else {
                    (*self.raw).functions
                },
                (*self.raw).numFunctions as usize,
            )
            .iter_mut()
            .find(|func| eq_cstr_str(func.name, function_name))
            .map(NonNull::from)
            .ok_or(Error::FunctionNotFound)?
        };
        Function::from_raw(self.rt, func).and_then(Function::compile)
    }

    /// Looks up a function by its index in this module.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * the index is out of bounds
    /// * the function has been found but the signature did not match
    pub fn function<Args, Ret>(&self, function_index: usize) -> Result<Function<'rt, Args, Ret>>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let func = unsafe {
            slice::from_raw_parts_mut(
                if (*self.raw).functions.is_null() {
                    NonNull::dangling().as_ptr()
                } else {
                    (*self.raw).functions
                },
                (*self.raw).numFunctions as usize,
            )
            .get(function_index)
            .map(NonNull::from)
            .ok_or(Error::FunctionNotFound)?
        };
        Function::from_raw(self.rt, func).and_then(Function::compile)
    }

    /// The name of this module.
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str((*self.raw).name) }
    }

    /// Links wasi to this module.
    #[cfg(feature = "wasi")]
    pub fn link_wasi(&mut self) -> Result<()> {
        unsafe { Error::from_ffi_res(ffi::m3_LinkWASI(self.raw)) }
    }
}

impl<'rt> Module<'rt> {
    pub(crate) fn from_raw(rt: &'rt Runtime, raw: ffi::IM3Module) -> Self {
        Module { raw, rt }
    }

    unsafe fn link_func_impl(&self, mut m3_func: NNM3Function, func: RawCall) -> Result<()> {
        let page = wasm3_priv::AcquireCodePageWithCapacity(self.rt.as_ptr(), 2);
        if page.is_null() {
            Error::from_ffi_res(ffi::m3Err_mallocFailedCodePage)
        } else {
            m3_func.as_mut().compiled = wasm3_priv::GetPagePC(page);
            m3_func.as_mut().module = self.raw;
            wasm3_priv::EmitWord_impl(page, crate::wasm3_priv::op_CallRawFunction as _);
            wasm3_priv::EmitWord_impl(page, func as _);

            wasm3_priv::ReleaseCodePage(self.rt.as_ptr(), page);
            Ok(())
        }
    }

    unsafe fn link_closure_impl<Args, Ret, F>(
        &self,
        mut m3_func: NNM3Function,
        closure: *mut F,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
        F: for<'cc> FnMut(CallContext<'cc>, Args) -> Ret + 'static,
    {
        unsafe extern "C" fn _impl<Args, Ret, F>(
            runtime: ffi::IM3Runtime,
            sp: ffi::m3stack_t,
            _mem: *mut cty::c_void,
            closure: *mut cty::c_void,
        ) -> *const cty::c_void
        where
            Args: crate::WasmArgs,
            Ret: crate::WasmType,
            F: for<'cc> FnMut(CallContext<'cc>, Args) -> Ret + 'static,
        {
            // use https://doc.rust-lang.org/std/primitive.pointer.html#method.offset_from once stable
            let stack_base = (*runtime).stack as ffi::m3stack_t;
            let stack_occupied =
                (sp as usize - stack_base as usize) / core::mem::size_of::<ffi::m3slot_t>();
            let stack = ptr::slice_from_raw_parts_mut(
                sp,
                (*runtime).numStackSlots as usize - stack_occupied,
            );

            let args = Args::pop_from_stack(stack);
            let context = CallContext::from_rt(NonNull::new_unchecked(runtime));
            let ret = (&mut *closure.cast::<F>())(context, args);
            ret.push_on_stack(stack.cast());
            ffi::m3Err_none as _
        }

        let page = wasm3_priv::AcquireCodePageWithCapacity(self.rt.as_ptr(), 3);
        if page.is_null() {
            Error::from_ffi_res(ffi::m3Err_mallocFailedCodePage)
        } else {
            m3_func.as_mut().compiled = wasm3_priv::GetPagePC(page);
            m3_func.as_mut().module = self.raw;
            wasm3_priv::EmitWord_impl(page, crate::wasm3_priv::op_CallRawFunctionEx as _);
            wasm3_priv::EmitWord_impl(page, _impl::<Args, Ret, F> as _);
            wasm3_priv::EmitWord_impl(page, closure.cast());

            wasm3_priv::ReleaseCodePage(self.rt.as_ptr(), page);
            Ok(())
        }
    }

    fn find_import_function(&self, module_name: &str, function_name: &str) -> Result<NNM3Function> {
        unsafe {
            slice::from_raw_parts_mut(
                if (*self.raw).functions.is_null() {
                    NonNull::dangling().as_ptr()
                } else {
                    (*self.raw).functions
                },
                (*self.raw).numFunctions as usize,
            )
            .iter_mut()
            .filter(|func| eq_cstr_str(func.import.moduleUtf8, module_name))
            .find(|func| eq_cstr_str(func.import.fieldUtf8, function_name))
            .map(NonNull::from)
            .ok_or(Error::FunctionNotFound)
        }
    }
}

#[test]
fn module_parse() {
    let env = Environment::new().expect("env alloc failure");
    let fib32 = [
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01,
        0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x66, 0x69, 0x62, 0x00, 0x00, 0x0a,
        0x1f, 0x01, 0x1d, 0x00, 0x20, 0x00, 0x41, 0x02, 0x49, 0x04, 0x40, 0x20, 0x00, 0x0f, 0x0b,
        0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6b, 0x10, 0x00, 0x6a,
        0x0f, 0x0b,
    ];
    let _ = Module::parse(&env, &fib32[..]).unwrap();
}
