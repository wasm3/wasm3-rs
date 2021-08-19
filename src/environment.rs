use alloc::boxed::Box;
use alloc::rc::Rc;

use core::ptr::NonNull;

use crate::error::{Error, Result};
use crate::module::ParsedModule;
use crate::runtime::Runtime;

#[derive(Debug)]
struct DropEnvironment(NonNull<ffi::M3Environment>);

impl Drop for DropEnvironment {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeEnvironment(self.0.as_ptr()) };
    }
}

/// An environment is required to construct [`Runtime`]s from.
#[derive(Debug, Clone)]
pub struct Environment(Rc<DropEnvironment>);

impl Environment {
    /// Creates a new environment.
    ///
    /// # Errors
    ///
    /// This function will error on memory allocation failure.
    #[inline]
    pub fn new() -> Result<Self> {
        unsafe { NonNull::new(ffi::m3_NewEnvironment()) }
            .ok_or_else(Error::malloc_error)
            .map(|raw| Environment(Rc::new(DropEnvironment(raw))))
    }

    /// Creates a new runtime with the given stack size in slots.
    ///
    /// # Errors
    ///
    /// This function will error on memory allocation failure.
    #[inline]
    pub fn create_runtime(&self, stack_size: u32) -> Result<Runtime> {
        Runtime::new(self, stack_size)
    }

    /// Parses a wasm module from raw bytes.
    #[inline]
    pub fn parse_module<TData: Into<Box<[u8]>>>(&self, bytes: TData) -> Result<ParsedModule> {
        ParsedModule::parse(self, bytes)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> ffi::IM3Environment {
        (self.0).0.as_ptr()
    }
}

impl core::cmp::Eq for Environment {}
impl core::cmp::PartialEq for Environment {
    fn eq(&self, &Environment(ref other): &Environment) -> bool {
        alloc::rc::Rc::ptr_eq(&self.0, other)
    }
}

#[test]
fn create_and_drop_env() {
    assert!(Environment::new().is_ok());
}
