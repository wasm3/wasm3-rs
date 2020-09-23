use alloc::boxed::Box;
use alloc::rc::Rc;
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
        })
    }

    /// Parses and loads a module from bytes.
    pub fn parse_and_load_module(self: &Rc<Self>, bytes: &[u8]) -> Result<Module> {
        Module::parse(&self.environment, bytes).and_then(|module| self.load_module(module))
    }

    /// Loads a parsed module returning the module if unsuccessful.
    ///
    /// # Errors
    ///
    /// This function will error if the module's environment differs from the one this runtime uses.
    pub fn load_module(self: &Rc<Self>, module: ParsedModule) -> Result<Module> {
        if &self.environment != module.environment() {
            Err(Error::ModuleLoadEnvMismatch)
        } else {
            Error::from_ffi_res(unsafe { ffi::m3_LoadModule(self.raw.as_ptr(), module.as_ptr()) })?;
            let raw = module.as_ptr();
            mem::forget(module);
            Ok(Module::from_raw(self.clone(), raw))
        }
    }

    /// Looks up a function by the given name in the loaded modules of this runtime.
    /// See [`Module::find_function`] for possible error cases.
    ///
    /// [`Module::find_function`]: ../module/struct.Module.html#method.find_function
    pub fn find_function<ARGS, RET>(self: &Rc<Self>, name: &str) -> Result<Function<ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        self.modules()
            .find_map(|module| match module.find_function::<ARGS, RET>(name) {
                res @ Ok(_) | res @ Err(Error::InvalidFunctionSignature) => Some(res),
                _ => None,
            })
            .unwrap_or(Err(Error::FunctionNotFound))
    }

    /// Searches for a module with the given name in the runtime's loaded modules.
    ///
    /// Using this over searching through [`Runtime::modules`] is a bit more efficient as it
    /// works on the underlying CStrings directly and doesn't require an upfront length calculation.
    ///
    /// [`Runtime::modules`]: struct.Runtime.html#method.modules
    pub fn find_module(self: &Rc<Self>, name: &str) -> Result<Module> {
        unsafe {
            let mut module = ptr::NonNull::new(self.raw.as_ref().modules);
            while let Some(raw_mod) = module {
                if eq_cstr_str(raw_mod.as_ref().name, name) {
                    return Ok(Module::from_raw(self.clone(), raw_mod.as_ptr()));
                }

                module = ptr::NonNull::new(raw_mod.as_ref().next);
            }
            Err(Error::ModuleNotFound)
        }
    }

    /// Returns an iterator over the runtime's loaded modules.
    pub fn modules<'rt>(self: &'rt Rc<Self>) -> impl Iterator<Item = Module> + 'rt {
        // pointer could get invalidated if modules can become unloaded
        // pushing new modules into the runtime while this iterator exists is fine as its backed by a linked list meaning it wont get invalidated.
        let mut module = unsafe { ptr::NonNull::new(self.raw.as_ref().modules) };
        core::iter::from_fn(move || {
            let next = unsafe { module.and_then(|module| ptr::NonNull::new(module.as_ref().next)) };
            mem::replace(&mut module, next).map(|raw| Module::from_raw(self.clone(), raw.as_ptr()))
        })
    }

    /// Resizes the number of allocatable pages to num_pages.
    ///
    /// # Errors
    ///
    /// This function will error out if it failed to resize memory allocation.
    pub fn resize_memory(&self, num_pages: u32) -> Result<()> {
        Error::from_ffi_res(unsafe { ffi::ResizeMemory(self.raw.as_ptr(), num_pages) })
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

    /// Returns the stack of this runtime.
    pub fn stack(&self) -> *const [ffi::m3slot_t] {
        unsafe {
            ptr::slice_from_raw_parts(
                self.raw.as_ref().stack.cast::<ffi::m3slot_t>(),
                self.raw.as_ref().numStackSlots as usize,
            )
        }
    }

    /// Returns the stack of this runtime.
    pub fn stack_mut(&self) -> *mut [ffi::m3slot_t] {
        unsafe {
            ptr::slice_from_raw_parts_mut(
                self.raw.as_ref().stack.cast::<ffi::m3slot_t>(),
                self.raw.as_ref().numStackSlots as usize,
            )
        }
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
