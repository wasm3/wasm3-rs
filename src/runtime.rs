use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::mem;
use core::pin::Pin;
use core::ptr::{self, NonNull};

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::Function;
use crate::module::{Module, ParsedModule};
use crate::utils::eq_cstr_str;

type PinnedAnyClosure = Pin<Box<dyn core::any::Any + 'static>>;

/// A runtime context for wasm3 modules.
#[derive(Debug)]
pub struct Runtime {
    raw: NonNull<ffi::M3Runtime>,
    environment: Environment,
    // holds all linked closures so that they properly get disposed of when runtime drops
    closure_store: UnsafeCell<Vec<PinnedAnyClosure>>,
    // holds all backing data of loaded modules as they have to be kept alive for the module's lifetime
    module_data: UnsafeCell<Vec<Box<[u8]>>>,
}

impl Runtime {
    /// Creates a new runtime with the given stack size in slots.
    ///
    /// # Errors
    ///
    /// This function will error on memory allocation failure.
    pub fn new(environment: &Environment, stack_size: u32) -> Result<Self> {
        unsafe {
            NonNull::new(ffi::m3_NewRuntime(
                environment.as_ptr(),
                stack_size,
                ptr::null_mut(),
            ))
        }
        .ok_or_else(Error::malloc_error)
        .map(|raw| Runtime {
            raw,
            environment: environment.clone(),
            closure_store: UnsafeCell::new(Vec::new()),
            module_data: UnsafeCell::new(Vec::new()),
        })
    }

    /// Parses and loads a module from bytes.
    pub fn parse_and_load_module<'rt, TData: Into<Box<[u8]>>>(
        &'rt self,
        bytes: TData,
    ) -> Result<Module<'rt>> {
        Module::parse(&self.environment, bytes).and_then(|module| self.load_module(module))
    }

    /// Loads a parsed module returning the module if unsuccessful.
    ///
    /// # Errors
    ///
    /// This function will error if the module's environment differs from the one this runtime uses.
    pub fn load_module<'rt>(&'rt self, module: ParsedModule) -> Result<Module<'rt>> {
        if &self.environment != module.environment() {
            Err(Error::ModuleLoadEnvMismatch)
        } else {
            let raw_mod = module.as_ptr();
            Error::from_ffi_res(unsafe { ffi::m3_LoadModule(self.raw.as_ptr(), raw_mod) })?;
            // SAFETY: Runtime isn't Send, therefor this access is single-threaded and kept alive only for the Vec::push call
            // as such this can not alias.
            unsafe { (*self.module_data.get()).push(module.take_data()) };

            Ok(Module::from_raw(self, raw_mod))
        }
    }

    /// Looks up a function by the given name in the loaded modules of this runtime.
    /// See [`Module::find_function`] for possible error cases.
    ///
    /// [`Module::find_function`]: ../module/struct.Module.html#method.find_function
    pub fn find_function<'rt, ARGS, RET>(&'rt self, name: &str) -> Result<Function<'rt, ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        self.modules()
            .find_map(|module| match module.find_function::<ARGS, RET>(name) {
                res @ (Ok(_) | Err(Error::InvalidFunctionSignature)) => Some(res),
                _ => None,
            })
            .unwrap_or(Err(Error::FunctionNotFound))
    }

    /// Returns the raw memory of this runtime.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub unsafe fn memory(&self) -> *const [u8] {
        let len = (*self.mallocated()).length as usize;
        let data = if len == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            self.mallocated().offset(1).cast()
        };
        ptr::slice_from_raw_parts(data, len)
    }

    /// Returns the raw memory of this runtime.
    ///
    /// # Safety
    ///
    /// The returned pointer may get invalidated when wasm function objects are called due to reallocations.
    pub unsafe fn memory_mut(&self) -> *mut [u8] {
        let len = (*self.mallocated()).length as usize;
        let data = if len == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            self.mallocated().offset(1).cast()
        };
        ptr::slice_from_raw_parts_mut(data, len)
    }
}

impl Runtime {
    pub(crate) unsafe fn mallocated(&self) -> *mut ffi::M3MemoryHeader {
        self.raw.as_ref().memory.mallocated
    }

    pub(crate) fn push_closure(&self, closure: PinnedAnyClosure) {
        unsafe { (*self.closure_store.get()).push(closure) };
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw.as_ptr()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeRuntime(self.raw.as_ptr()) };
    }
}

#[test]
fn create_and_drop_rt() {
    let env = Environment::new().expect("env alloc failure");
    assert!(Runtime::new(&env, 1024 * 64).is_ok());
}
