use core::marker::PhantomData;
use core::ptr;
use core::slice;

use crate::environment::Environment;
use crate::error::{Error, Result};
use crate::function::{Function, RawCall};
use crate::runtime::Runtime;
use crate::utils::eq_cstr_str;

pub struct ParsedModule<'env> {
    raw: ffi::IM3Module,
    _pd: PhantomData<&'env Environment>,
}

impl<'env> ParsedModule<'env> {
    pub fn parse(environment: &'env Environment, bytes: &[u8]) -> Result<Self> {
        assert!(bytes.len() <= !0u32 as usize);
        let mut module = ptr::null_mut();
        let res = unsafe {
            ffi::m3_ParseModule(
                environment.as_ptr(),
                &mut module,
                bytes.as_ptr(),
                bytes.len() as u32,
            )
        };
        Error::from_ffi_res(res).map(|_| ParsedModule {
            raw: module,
            _pd: PhantomData,
        })
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Module {
        self.raw
    }
}

impl Drop for ParsedModule<'_> {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeModule(self.raw) };
    }
}

// needs no drop as loaded modules will be cleaned up by the runtime
pub struct Module<'env, 'rt> {
    raw: ffi::IM3Module,
    rt: &'rt Runtime<'env>,
}

impl<'env, 'rt> Module<'env, 'rt> {
    pub(crate) fn from_raw(rt: &'rt Runtime<'env>, raw: ffi::IM3Module) -> Self {
        Module { raw, rt }
    }

    #[inline]
    pub fn parse(environment: &'env Environment, bytes: &[u8]) -> Result<ParsedModule<'env>> {
        ParsedModule::parse(environment, bytes)
    }

    pub fn link_function<ARGS, RET>(
        &mut self,
        module_name: &str,
        function_name: &str,
        f: RawCall,
    ) -> Result<()>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        let func = self.find_import_function(module_name, function_name)?;
        if Function::<'_, '_, ARGS, RET>::validate_sig(func) {
            unsafe { self.link_func_impl(func, f) }
            Ok(())
        } else {
            Err(Error::InvalidFunctionSignature)
        }
    }

    unsafe fn link_func_impl(&self, m3_func: ffi::IM3Function, func: RawCall) {
        let page = ffi::AcquireCodePageWithCapacity(self.rt.as_ptr(), 2);
        if page.is_null() {
            panic!("oom")
        } else {
            (*m3_func).compiled = ffi::GetPagePC(page);
            (*m3_func).module = self.raw;
            ffi::EmitWord_impl(page, ffi::op_CallRawFunction as _);
            ffi::EmitWord_impl(page, func as _);

            ffi::ReleaseCodePage(self.rt.as_ptr(), page);
        }
    }

    fn find_import_function(
        &self,
        module_name: &str,
        function_name: &str,
    ) -> Result<ffi::IM3Function> {
        if let Some(func) = unsafe {
            slice::from_raw_parts_mut(
                if (*self.raw).functions.is_null() {
                    ptr::NonNull::dangling().as_ptr()
                } else {
                    (*self.raw).functions
                },
                (*self.raw).numFunctions as usize,
            )
            .iter_mut()
            .filter(|func| eq_cstr_str(func.import.moduleUtf8, module_name))
            .find(|func| eq_cstr_str(func.import.fieldUtf8, function_name))
        } {
            Ok(func)
        } else {
            Err(Error::FunctionNotFound)
        }
    }

    pub fn find_function<ARGS, RET>(
        &self,
        function_name: &str,
    ) -> Result<Function<'env, 'rt, ARGS, RET>>
    where
        ARGS: crate::WasmArgs,
        RET: crate::WasmType,
    {
        if let Some(func) = unsafe {
            slice::from_raw_parts_mut(
                if (*self.raw).functions.is_null() {
                    ptr::NonNull::dangling().as_ptr()
                } else {
                    (*self.raw).functions
                },
                (*self.raw).numFunctions as usize,
            )
            .iter_mut()
            .find(|func| eq_cstr_str(func.name, function_name))
        } {
            Function::from_raw(self.rt, func).and_then(Function::compile)
        } else {
            Err(Error::FunctionNotFound)
        }
    }

    #[cfg(feature = "wasi")]
    pub fn link_wasi(&mut self) {
        unsafe { ffi::m3_LinkWASI(self.raw) };
    }

    pub fn link_libc(&mut self) {
        unsafe { ffi::m3_LinkLibC(self.raw) };
    }
}

#[test]
fn module_parse() {
    let env = Environment::new();
    let fib32 = [
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01,
        0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x66, 0x69, 0x62, 0x00, 0x00, 0x0a,
        0x1f, 0x01, 0x1d, 0x00, 0x20, 0x00, 0x41, 0x02, 0x49, 0x04, 0x40, 0x20, 0x00, 0x0f, 0x0b,
        0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6b, 0x10, 0x00, 0x6a,
        0x0f, 0x0b,
    ];
    let _ = Module::parse(&env, &fib32[..]).unwrap();
}
