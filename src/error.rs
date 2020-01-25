use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    Wasm3(&'static str),
    InvalidFunctionSignature,
}

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        unsafe {
            if ptr.is_null() {
                Ok(())
            } else {
                Err(Error::Wasm3(
                    std::ffi::CStr::from_ptr(ptr).to_str().unwrap(),
                ))
            }
        }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Wasm3(msg) => write!(f, "{}", msg),
            Error::InvalidFunctionSignature => {
                write!(f, "the found function had an unexpected signature")
            }
        }
    }
}
