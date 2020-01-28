use core::fmt;

use crate::utils::cstr_to_str;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Wasm3(&'static str),
    InvalidFunctionSignature,
    FunctionNotFound,
}

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        if ptr.is_null() {
            Ok(())
        } else {
            Err(Error::Wasm3(unsafe { cstr_to_str(ptr) }))
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
