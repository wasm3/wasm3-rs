use std::ffi::CString;
use std::marker::PhantomData;

use crate::error::{Error, Result};
use crate::runtime::Runtime;

pub struct Function<'env, 'rt>(ffi::IM3Function, PhantomData<&'rt Runtime<'env>>);

impl<'env, 'rt> Function<'env, 'rt> {
    #[inline]
    pub(crate) fn from_raw(_runtime: &'rt Runtime<'env>, raw: ffi::IM3Function) -> Self {
        Function(raw, PhantomData)
    }

    pub fn call(&self) -> Result<()> {
        unsafe { Error::from_ffi_res(ffi::m3_Call(self.0)) }
    }

    pub fn call_args(&self, args: &[&str]) -> Result<()> {
        assert!(args.len() <= !0u32 as usize);
        let args: Vec<_> = args.iter().map(|&arg| CString::new(arg).unwrap()).collect();
        let args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();
        unsafe {
            Error::from_ffi_res(ffi::m3_CallWithArgs(
                self.0,
                args.len() as u32,
                args.as_ptr(),
            ))
        }
    }
}
