use core::fmt;
use core::str;

use crate::bytes_till_null;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Wasm3(&'static str),
    InvalidFunctionSignature,
    FunctionNotFound,
}

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        unsafe {
            if ptr.is_null() {
                Ok(())
            } else {
                Err(Error::Wasm3(str::from_utf8_unchecked(bytes_till_null(ptr))))
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Wasm3(msg) => write!(f, "{}", msg),
            Error::InvalidFunctionSignature => {
                write!(f, "the found function had an unexpected signature")
            }
            Error::FunctionNotFound => write!(f, "the function could not be found"),
        }
    }
}
