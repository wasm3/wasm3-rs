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

    pub(crate) unsafe fn mallocated(&self) -> *mut ffi::M3MemoryHeader {
        (*self.raw).memory.mallocated
    }

    pub(crate) fn has_errored(&self) -> bool {
        unsafe { !(*self.raw).runtimeError.is_null() }
    }

    pub fn find_function<'rt, ARGS, RET>(
        &'rt self,
        name: &str,
    ) -> Result<Function<'env, 'rt, ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        unsafe {
            let mut function = ptr::null_mut();
            let name = CString::new(name).unwrap();
            Error::from_ffi_res(ffi::m3_FindFunction(&mut function, self.raw, name.as_ptr()))
                .and_then(|_| Function::from_raw(self, function))
        }
    }

    #[inline]
    pub fn print_info(&self) {
        unsafe { ffi::m3_PrintRuntimeInfo(self.raw) };
    }
    // FIXME: The following three functions are unsound, cause a function call can mutate all of these slices
    // on the c/wasm side while someone could possible hold a reference to these still

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

    pub fn stack(&self) -> &[u64] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.raw).stack as ffi::m3stack_t,
                (*self.raw).numStackSlots as usize,
            )
        }
    }

    // FIXME: Unsound due to aliasing, should use ref counting for this?
    pub fn stack_mut(&self) -> &mut [u64] {
        unsafe {
            std::slice::from_raw_parts_mut(
                (*self.raw).stack as ffi::m3stack_t,
                (*self.raw).numStackSlots as usize,
            )
        }
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
