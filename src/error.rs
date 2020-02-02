use core::cmp;
use core::fmt;

use crate::utils::cstr_to_str;

pub type Result<T> = core::result::Result<T, Error>;

pub type TrappedResult<T> = core::result::Result<T, Trap>;
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    OutOfBoundsMemoryAccess,
    DivisionByZero,
    IntegerOverflow,
    IntegerConversion,
    IndirectCallTypeMismatch,
    TableIndexOutOfRange,
    Exit,
    Abort,
    Unreachable,
    StackOverflow,
}

impl Trap {
    pub fn as_ptr(self) -> ffi::M3Result {
        unsafe {
            match self {
                Trap::OutOfBoundsMemoryAccess => ffi::m3Err_trapOutOfBoundsMemoryAccess,
                Trap::DivisionByZero => ffi::m3Err_trapDivisionByZero,
                Trap::IntegerOverflow => ffi::m3Err_trapIntegerOverflow,
                Trap::IntegerConversion => ffi::m3Err_trapIntegerConversion,
                Trap::IndirectCallTypeMismatch => ffi::m3Err_trapIndirectCallTypeMismatch,
                Trap::TableIndexOutOfRange => ffi::m3Err_trapTableIndexOutOfRange,
                Trap::Exit => ffi::m3Err_trapExit,
                Trap::Abort => ffi::m3Err_trapAbort,
                Trap::Unreachable => ffi::m3Err_trapUnreachable,
                Trap::StackOverflow => ffi::m3Err_trapStackOverflow,
            }
        }
    }
}

impl cmp::PartialEq<Wasm3Error> for Trap {
    fn eq(&self, &Wasm3Error(err): &Wasm3Error) -> bool {
        self.as_ptr() == err
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Trap {}
impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(unsafe { cstr_to_str(self.as_ptr()) }, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Wasm3Error(*const cty::c_char);

impl Wasm3Error {
    pub fn is_trap(self, trap: Trap) -> bool {
        trap.as_ptr() == self.0
    }
}

impl cmp::PartialEq<Trap> for Wasm3Error {
    fn eq(&self, trap: &Trap) -> bool {
        trap.as_ptr() == self.0
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Wasm3Error {}
impl fmt::Display for Wasm3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(unsafe { cstr_to_str(self.0) }, f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Wasm3(Wasm3Error),
    InvalidFunctionSignature,
    FunctionNotFound,
    ModuleNotFound,
}

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        if ptr.is_null() {
            Ok(())
        } else {
            Err(Error::Wasm3(Wasm3Error(ptr)))
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Wasm3(err) => fmt::Display::fmt(err, f),
            Error::InvalidFunctionSignature => {
                write!(f, "the found function had an unexpected signature")
            }
            Error::FunctionNotFound => write!(f, "the function could not be found"),
            Error::ModuleNotFound => write!(f, "the module could not be found"),
        }
    }
}
