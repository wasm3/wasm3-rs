use std::ffi::CString;
use std::mem;
use std::ptr;
use std::slice;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::Function;
use crate::module::Module;

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

    pub fn parse_and_load_module(&self, bytes: &[u8]) -> Result<()> {
        Module::parse(self.environment, bytes)
            .and_then(|module| self.load_module(module).map_err(|(_, err)| err))
    }

    pub fn load_module(
        &self,
        module: Module<'env>,
    ) -> std::result::Result<(), (Module<'env>, Error)> {
        if let Err(err) =
            unsafe { Error::from_ffi_res(ffi::m3_LoadModule(self.raw, module.as_ptr())) }
        {
            Err((module, err))
        } else {
            mem::forget(module);
            Ok(())
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
    ) -> Option<Function<'env, 'rt, ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        unsafe {
            let mut function = ptr::null_mut();
            let name = CString::new(name).unwrap();
            Error::from_ffi_res(ffi::m3_FindFunction(&mut function, self.raw, name.as_ptr()))
                .and_then(|_| Function::from_raw(self, function))
                .ok()
        }
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
                std::ptr::NonNull::dangling().as_ptr()
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
        std::slice::from_raw_parts(
            (*self.raw).stack as ffi::m3stack_t,
            (*self.raw).numStackSlots as usize,
        )
    }

    /// # Safety
    /// This function is unsafe because it allows aliasing to happen.
    /// The underlying memory may change if a runtimes exposed function is called.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn stack_mut(&self) -> &mut [u64] {
        std::slice::from_raw_parts_mut(
            (*self.raw).stack as ffi::m3stack_t,
            (*self.raw).numStackSlots as usize,
        )
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
