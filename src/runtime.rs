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

    pub fn parse_and_load_module(&mut self, bytes: &[u8]) -> Result<()> {
        Module::parse(self.environment, bytes)
            .and_then(|module| self.load_module(module).map_err(|(_, err)| err))
    }

    pub fn load_module(
        &mut self,
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

    pub fn memory(&self) -> &[u8] {
        unsafe {
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
    }

    pub fn find_function<'rt>(&'rt self, name: &str) -> Result<Function<'env, 'rt>> {
        unsafe {
            let mut function = ptr::null_mut();
            let name = CString::new(name).unwrap();
            Error::from_ffi_res(ffi::m3_FindFunction(&mut function, self.raw, name.as_ptr()))
                .map(|_| Function::from_raw(self, function))
        }
    }

    #[inline]
    pub fn print_info(&self) {
        unsafe { ffi::m3_PrintRuntimeInfo(self.raw) };
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
