use std::marker::PhantomData;
use std::ptr;

use crate::environment::Environment;
use crate::error::{Error, Result};

#[derive(Clone)]
pub struct Module<'env>(ffi::IM3Module, PhantomData<&'env Environment>);

impl<'env> Module<'env> {
    pub fn parse(environment: &'env Environment, bytes: &[u8]) -> Result<Self> {
        assert!(bytes.len() <= !0u32 as usize);
        unsafe {
            let mut module = ptr::null_mut();
            let res = ffi::m3_ParseModule(
                environment.as_ptr(),
                &mut module,
                bytes.as_ptr(),
                bytes.len() as u32,
            );
            Error::from_ffi_res(res).map(|_| Module(module, PhantomData))
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> ffi::IM3Module {
        self.0
    }
}

impl Drop for Module<'_> {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeModule(self.0) };
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
