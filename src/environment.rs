use alloc::rc::Rc;

use crate::error::Result;
use crate::module::ParsedModule;
use crate::runtime::Runtime;

#[derive(Debug)]
struct DropEnvironment(ffi::IM3Environment);

impl Drop for DropEnvironment {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeEnvironment(self.0) };
    }
}

#[derive(Debug, Clone)]
pub struct Environment(Rc<DropEnvironment>);

impl Environment {
    #[inline]
    pub fn new() -> Self {
        unsafe { Environment(Rc::new(DropEnvironment(ffi::m3_NewEnvironment()))) }
    }

    #[inline]
    pub fn create_runtime(&self, stack_size: u32) -> Result<Runtime> {
        Runtime::new(self, stack_size)
    }

    #[inline]
    pub fn parse_module(&self, bytes: &[u8]) -> Result<ParsedModule> {
        ParsedModule::parse(self, bytes)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> ffi::IM3Environment {
        (self.0).0
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
    let _ = Environment::new();
}
