//! Error related functionality of wasm3.
use core::cmp;
use core::fmt;

use crate::utils::cstr_to_str;

lazy_static::lazy_static! {
/// Out of bounds memory access error static str
pub static ref OUT_OF_BOUNDS_MEMORY_ACCESS_STR:&'static str= unsafe { cstr_to_str(ffi::m3Err_trapOutOfBoundsMemoryAccess) };
/// Division by zero error static str
pub static ref DIVISION_BY_ZERO_STR:&'static str= unsafe { cstr_to_str(ffi::m3Err_trapDivisionByZero) };
/// Integer overflow error static str
pub static ref INTEGER_OVERFLOW_STR:&'static str= unsafe { cstr_to_str(ffi::m3Err_trapIntegerOverflow) };
/// Integer conversion error static str
pub static ref INTEGER_CONVERSION_ST:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapIntegerConversion) };
/// Indirect call type mismatch error static str
pub static ref INDIRECT_CALL_TYPE_MISMATCH_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapIndirectCallTypeMismatch) };
/// Table index out of range error static str
pub static ref TABLE_INDEX_OUT_OF_RANGE_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapTableIndexOutOfRange) };
/// Exit error static str
pub static ref EXIT_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapExit) };
/// Abort error static str
pub static ref ABORT_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapAbort) };
/// Unreachable error static str
pub static ref UNREACHABLE_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapUnreachable) };
/// Stack overflow error static str
pub static ref STACK_OVERFLOW_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_trapStackOverflow) };

/// function lookup failed error static str
pub static ref FUNCTION_LOOKUP_FAILED_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_functionLookupFailed) };
/// malloc failed error static str
pub static ref MALLOC_FAILED_STR:&'static str=unsafe { cstr_to_str(ffi::m3Err_mallocFailed) };

}
/// Result alias that uses [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
/// Result alias that uses [`Trap`].
pub type TrappedResult<T> = core::result::Result<T, Trap>;

/// A wasm trap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    /// Out of bounds memory access
    OutOfBoundsMemoryAccess,
    /// Division by zero
    DivisionByZero,
    /// Integer overflow
    IntegerOverflow,
    /// Integer conversion
    IntegerConversion,
    /// Indirect call type mismatch
    IndirectCallTypeMismatch,
    /// Table index out of range
    TableIndexOutOfRange,
    /// Exit
    Exit,
    /// Abort
    Abort,
    /// Unreachable
    Unreachable,
    /// Stack overflow
    StackOverflow,
}


impl Trap {
    #[doc(hidden)]
    pub fn as_str(self) -> &'static str {
        match self {
            Trap::OutOfBoundsMemoryAccess => &OUT_OF_BOUNDS_MEMORY_ACCESS_STR,
            Trap::DivisionByZero =>  &DIVISION_BY_ZERO_STR,
            Trap::IntegerOverflow =>  &INTEGER_OVERFLOW_STR,
            Trap::IntegerConversion =>  &INTEGER_CONVERSION_ST,
            Trap::IndirectCallTypeMismatch =>  &INDIRECT_CALL_TYPE_MISMATCH_STR,
            Trap::TableIndexOutOfRange =>  &TABLE_INDEX_OUT_OF_RANGE_STR,
            Trap::Exit =>  &EXIT_STR,
            Trap::Abort =>  &ABORT_STR,
            Trap::Unreachable =>  &UNREACHABLE_STR,
            Trap::StackOverflow =>  &STACK_OVERFLOW_STR,
        }
    }
    #[doc(hidden)]
    pub (crate) fn as_ptr(self)->* const i8{
        self.as_str().as_ptr().cast()
    }
}

impl cmp::PartialEq<Wasm3Error> for Trap {
    fn eq(&self, &Wasm3Error(msg): &Wasm3Error) -> bool {
        self.as_str() == msg
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Trap {}
impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}


/// Error returned by wasm3.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Wasm3Error(&'static str);

impl Wasm3Error {
    /// Check whether this error is the specified trap.
    pub fn is_trap(self, trap: Trap) -> bool {
        trap == self
    }
}

impl cmp::PartialEq<Trap> for Wasm3Error {
    fn eq(&self, trap: &Trap) -> bool {
        trap.as_str() == self.0
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Wasm3Error {}
impl fmt::Debug for Wasm3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}
impl fmt::Display for Wasm3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

/// Error returned by wasm3-rs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An error originating from wasm3 itself may or may not be a trap.
    Wasm3(Wasm3Error),
    /// A function has been found but its signature didn't match.
    InvalidFunctionSignature,
    /// The specified function could not be found.
    FunctionNotFound,
    /// The specified module could not be found.
    ModuleNotFound,
    /// The modules environment did not match the runtime's environment.
    ModuleLoadEnvMismatch,
}

impl Error {
    pub(crate) fn from_ffi_res(ptr: ffi::M3Result) -> Result<()> {
        if ptr.is_null() {
            Ok(())
        } else if unsafe { ptr == ffi::m3Err_functionLookupFailed } {
            Err(Error::Wasm3(Wasm3Error( &FUNCTION_LOOKUP_FAILED_STR)))
        } else {
            Err(Error::Wasm3(Wasm3Error(unsafe { cstr_to_str(ptr) })))
        }
    }

    pub(crate) fn malloc_error() -> Self {
        Error::Wasm3(Wasm3Error( &MALLOC_FAILED_STR))
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
            Error::ModuleLoadEnvMismatch => {
                write!(f, "the module and runtime environments were not the same")
            }
        }
    }
}
