use core::mem;
use core::ptr;
use core::slice;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::Function;
use crate::module::{Module, ParsedModule};
use crate::utils::eq_cstr_str;

#[derive(Debug)]
pub struct Runtime<'env> {
    raw: ffi::IM3Runtime,
    environment: &'env Environment,
}

impl<'env> Runtime<'env> {
    pub fn new(environment: &'env Environment, stack_size: u32) -> Self {
        unsafe {
            Runtime {
                raw: ffi::m3_NewRuntime(environment.as_ptr(), stack_size, ptr::null_mut()),
                environment,
            }
        }
    }

    pub fn parse_and_load_module<'rt>(&'rt self, bytes: &[u8]) -> Result<Module<'env, 'rt>> {
        Module::parse(self.environment, bytes)
            .and_then(|module| self.load_module(module).map_err(|(_, err)| err))
    }

    pub fn load_module<'rt>(
        &'rt self,
        module: ParsedModule<'env>,
    ) -> core::result::Result<Module<'env, 'rt>, (ParsedModule<'env>, Error)> {
        if let Err(err) =
            Error::from_ffi_res(unsafe { ffi::m3_LoadModule(self.raw, module.as_ptr()) })
        {
            Err((module, err))
        } else {
            let raw = module.as_ptr();
            mem::forget(module);
            Ok(Module::from_raw(self, raw))
        }
    }

    pub(crate) unsafe fn mallocated(&self) -> *mut ffi::M3MemoryHeader {
        (*self.raw).memory.mallocated
    }

    pub(crate) fn rt_error(&self) -> Result<()> {
        unsafe { Error::from_ffi_res((*self.raw).runtimeError) }
    }

    pub fn find_function<'rt, ARGS, RET>(
        &'rt self,
        name: &str,
    ) -> Result<Function<'env, 'rt, ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        self.modules()
            .find_map(|module| match module.find_function::<ARGS, RET>(name) {
                res @ Ok(_) => Some(res),
                res @ Err(Error::InvalidFunctionSignature) => Some(res),
                _ => None,
            })
            .unwrap_or(Err(Error::FunctionNotFound))
    }

    /// Using this over searching through [`modules`] is a bit more efficient.
    pub fn find_module<'rt>(&'rt self, name: &str) -> Result<Module<'env, 'rt>> {
        unsafe {
            let mut module = ptr::NonNull::new((*self.raw).modules);
            while let Some(raw_mod) = module {
                if eq_cstr_str(raw_mod.as_ref().name, name) {
                    return Ok(Module::from_raw(self, raw_mod.as_ptr()));
                }
                module = ptr::NonNull::new(raw_mod.as_ref().next);
            }
            Err(Error::ModuleNotFound)
        }
    }

    pub fn modules<'rt>(&'rt self) -> impl Iterator<Item = Module<'env, 'rt>> + 'rt {
        // pointer could get invalidated if modules can become unloaded
        let mut module = unsafe { ptr::NonNull::new((*self.raw).modules) };
        core::iter::from_fn(move || {
            let next = unsafe { module.and_then(|module| ptr::NonNull::new(module.as_ref().next)) };
            mem::replace(&mut module, next).map(|raw| Module::from_raw(self, raw.as_ptr()))
        })
    }

    #[inline]
    pub fn print_info(&self) {
        unsafe { ffi::m3_PrintRuntimeInfo(self.raw) };
    }

    /// # Safety
    /// This function is unsafe because it allows aliasing to happen.
    /// The underlying memory may change if a runtimes exposed function is called.
    pub unsafe fn memory(&self) -> &[u8] {
        let mut size = 0;
        let ptr = ffi::m3_GetMemory(self.raw, &mut size, 0);
        slice::from_raw_parts(
            if size == 0 || ptr.is_null() {
                ptr::NonNull::dangling().as_ptr()
            } else {
                ptr
            },
            size as usize,
        )
    }

    /// # Safety
    /// This function is unsafe because it allows aliasing to happen.
    /// The underlying memory may change if a runtimes exposed function is called.
    pub unsafe fn stack(&self) -> &[u64] {
        slice::from_raw_parts(
            (*self.raw).stack as ffi::m3stack_t,
            (*self.raw).numStackSlots as usize,
        )
    }

    /// # Safety
    /// This function is unsafe because it allows aliasing to happen.
    /// The underlying memory may change if a runtimes exposed function is called.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn stack_mut(&self) -> &mut [u64] {
        slice::from_raw_parts_mut(
            (*self.raw).stack as ffi::m3stack_t,
            (*self.raw).numStackSlots as usize,
        )
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Runtime {
        self.raw
    }
}

impl<'env> Drop for Runtime<'env> {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeRuntime(self.raw) };
    }
}

#[test]
fn create_and_drop_rt() {
    let env = Environment::new();
    let _ = Runtime::new(&env, 1024 * 64);
}
