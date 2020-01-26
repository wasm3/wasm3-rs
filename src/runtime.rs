use core::mem;
use core::ptr;
use core::slice;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::Function;
use crate::module::{Module, ParsedModule};

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
            unsafe { Error::from_ffi_res(ffi::m3_LoadModule(self.raw, module.as_ptr())) }
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
        let mut module = unsafe { (*self.raw).modules };
        while !module.is_null() {
            match Module::from_raw(self, module).find_function::<ARGS, RET>(name) {
                res @ Ok(_) => return res,
                res @ Err(Error::InvalidFunctionSignature) => return res,
                _ => module = unsafe { (*module).next },
            }
        }
        Err(Error::FunctionNotFound)
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
            if size == 0 {
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
