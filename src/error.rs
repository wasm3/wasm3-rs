use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Error(&'static str);

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        // cant match on the variants cause they are mut statics for bindgen
        unsafe {
            if ptr == ffi::m3Err_none {
                Ok(())
            } else {
                Err(Error(std::ffi::CStr::from_ptr(ptr).to_str().unwrap()))
            }
        }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
