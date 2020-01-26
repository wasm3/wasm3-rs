use crate::error::Result;
use crate::module::ParsedModule;
use crate::runtime::Runtime;

pub struct Environment(ffi::IM3Environment);

impl Environment {
    #[inline]
    pub fn new() -> Self {
        unsafe { Environment(ffi::m3_NewEnvironment()) }
    }

    #[inline]
    pub fn create_runtime(&self, stack_size: u32) -> Runtime {
        Runtime::new(self, stack_size)
    }

    #[inline]
    pub fn parse_module<'env>(&'env self, bytes: &[u8]) -> Result<ParsedModule<'env>> {
        ParsedModule::parse(self, bytes)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> ffi::IM3Environment {
        self.0
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeEnvironment(self.as_ptr()) };
    }
}

#[test]
fn create_and_drop_env() {
    let _ = Environment::new();
}
