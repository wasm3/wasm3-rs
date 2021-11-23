use alloc::boxed::Box;
use alloc::vec::Vec;

use core::mem;
use core::ptr::{self, NonNull};

use crate::environment::Environment;
use crate::error::{Error, Result, Trap};
use crate::function::{CallContext, Function, RawCall};
use crate::runtime::Runtime;
use crate::utils::{cstr_to_str, str_to_cstr_owned};

#[derive(Debug)]
struct DropModule(NonNull<ffi::M3Module>);

impl Drop for DropModule {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeModule(self.0.as_ptr()) };
    }
}

/// A parsed module which can be loaded into a [`Runtime`].
pub struct ParsedModule {
    data: Box<[u8]>,
    raw: DropModule,
    env: Environment,
}

impl ParsedModule {
    /// Parses a wasm module from raw bytes.
    pub fn parse<TData: Into<Box<[u8]>>>(env: &Environment, data: TData) -> Result<Self> {
        let data = data.into();
        assert!(data.len() <= !0u32 as usize);
        let mut module = ptr::null_mut();
        let res = unsafe {
            ffi::m3_ParseModule(env.as_ptr(), &mut module, data.as_ptr(), data.len() as u32)
        };
        Error::from_ffi_res(res)?;
        let module = NonNull::new(module)
            .expect("module pointer is non-null after m3_ParseModule if result is not error");
        Ok(ParsedModule {
            data,
            raw: DropModule(module),
            env: env.clone(),
        })
    }

    pub(crate) fn as_ptr(&self) -> ffi::IM3Module {
        self.raw.0.as_ptr()
    }

    pub(crate) fn take_data(self) -> Box<[u8]> {
        let ParsedModule {
            data,
            raw,
            env: _env,
        } = self;
        mem::forget(raw);
        data
    }

    /// The environment this module was parsed in.
    pub fn environment(&self) -> &Environment {
        &self.env
    }
}

/// A loaded module belonging to a specific runtime. Allows for linking and looking up functions.
// needs no drop as loaded modules will be cleaned up by the runtime
pub struct Module<'rt> {
    raw: ffi::IM3Module,
    rt: &'rt Runtime,
}

impl<'rt> Module<'rt> {
    /// Parses a wasm module from raw bytes.
    #[inline]
    pub fn parse<TData: Into<Box<[u8]>>>(
        environment: &Environment,
        bytes: TData,
    ) -> Result<ParsedModule> {
        ParsedModule::parse(environment, bytes)
    }

    /// Links the given function to the corresponding module and function name.
    /// This allows linking a more verbose function, as it gets access to the unsafe
    /// runtime parts. For easier use the [`make_func_wrapper`] should be used to create
    /// the unsafe facade for your function that then can be passed to this.
    ///
    /// For a simple API see [`link_closure`] which takes a closure instead.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    ///
    /// [`link_closure`]: #method.link_closure
    pub fn link_function<Args, Ret>(
        &mut self,
        module_name: &str,
        function_name: &str,
        f: RawCall,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let module_name_cstr = str_to_cstr_owned(module_name);
        let function_name_cstr = str_to_cstr_owned(function_name);
        let signature = function_signature::<Args, Ret>();

        let result = unsafe {
            ffi::m3_LinkRawFunction(
                self.raw,
                module_name_cstr.as_ptr(),
                function_name_cstr.as_ptr(),
                signature.as_ptr(),
                Some(f),
            )
        };
        Error::from_ffi_res(result)
    }

    /// Links the given closure to the corresponding module and function name.
    /// This boxes the closure and therefor requires a heap allocation.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn link_closure<Args, Ret, F>(
        &mut self,
        module_name: &str,
        function_name: &str,
        closure: F,
    ) -> Result<()>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
        F: for<'cc> FnMut(CallContext<'cc>, Args) -> core::result::Result<Ret, Trap> + 'static,
    {
        unsafe extern "C" fn trampoline<Args, Ret, F>(
            runtime: ffi::IM3Runtime,
            ctx: ffi::IM3ImportContext,
            sp: *mut u64,
            _mem: *mut cty::c_void,
        ) -> *const cty::c_void
        where
            Args: crate::WasmArgs,
            Ret: crate::WasmType,
            F: for<'cc> FnMut(CallContext<'cc>, Args) -> core::result::Result<Ret, Trap> + 'static,
        {
            let runtime = NonNull::new(runtime)
                .expect("wasm3 calls imported functions with non-null runtime");
            let ctx = NonNull::new(ctx)
                .expect("wasm3 calls imported functions with non-null import context");
            let mut closure = NonNull::new(ctx.as_ref().userdata as *mut F)
                .expect("userdata passed to m3_LinkRawFunctionEx is non-null");

            let args = Args::pop_from_stack(sp.add(Ret::SIZE_IN_SLOT_COUNT));
            let ret = closure.as_mut()(CallContext::from_rt(runtime), args);
            let result = match ret {
                Ok(ret) => {
                    ret.push_on_stack(sp);
                    ffi::m3Err_none
                }
                Err(trap) => trap.as_ptr(),
            };
            result as *const cty::c_void
        }

        let module_name_cstr = str_to_cstr_owned(module_name);
        let function_name_cstr = str_to_cstr_owned(function_name);
        let signature = function_signature::<Args, Ret>();

        let mut closure = Box::pin(closure);
        let result = unsafe {
            ffi::m3_LinkRawFunctionEx(
                self.raw,
                module_name_cstr.as_ptr(),
                function_name_cstr.as_ptr(),
                signature.as_ptr(),
                Some(trampoline::<Args, Ret, F>),
                closure.as_mut().get_unchecked_mut() as *mut F as *const cty::c_void,
            )
        };
        Error::from_ffi_res(result)?;
        self.rt.push_closure(closure);
        Ok(())
    }

    /// Looks up a function by the given name in this module.
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations:
    ///
    /// * a memory allocation failed
    /// * no function by the given name in the given module could be found
    /// * the function has been found but the signature did not match
    pub fn find_function<Args, Ret>(&self, function_name: &str) -> Result<Function<'rt, Args, Ret>>
    where
        Args: crate::WasmArgs,
        Ret: crate::WasmType,
    {
        let function = self.rt.find_function(function_name)?;
        match function.module() {
            Some(module) if module.raw == self.raw => Ok(function),
            _ => Err(Error::FunctionNotFound),
        }
    }

    /// The name of this module.
    pub fn name(&self) -> &str {
        unsafe { cstr_to_str(ffi::m3_GetModuleName(self.raw)) }
    }

    /// Links wasi to this module.
    #[cfg(feature = "wasi")]
    pub fn link_wasi(&mut self) -> Result<()> {
        unsafe { Error::from_ffi_res(ffi::m3_LinkWASI(self.raw)) }
    }
}

impl<'rt> Module<'rt> {
    pub(crate) fn from_raw(rt: &'rt Runtime, raw: ffi::IM3Module) -> Self {
        Module { raw, rt }
    }
}

fn function_signature<Args, Ret>() -> Vec<cty::c_char>
where
    Args: crate::WasmArgs,
    Ret: crate::WasmType,
{
    let mut signature = <Vec<cty::c_char>>::new();
    signature.push(Ret::SIGNATURE as cty::c_char);
    signature.push(b'(' as cty::c_char);
    Args::append_signature(&mut signature);
    signature.push(b')' as cty::c_char);
    signature.push(b'\0' as cty::c_char);
    signature
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TrappedResult;
    use crate::make_func_wrapper;

    make_func_wrapper!(mul_u32_and_f32_wrap: mul_u32_and_f32(a: u32, b: f32) -> f64);
    fn mul_u32_and_f32(a: u32, b: f32) -> f64 {
        (a as f64) * (b as f64)
    }

    make_func_wrapper!(hello_wrap: hello() -> TrappedResult<()>);
    fn hello() -> TrappedResult<()> {
        Ok(())
    }

    const TEST_BIN: &[u8] = include_bytes!("../tests/wasm_test_bins/wasm_test_bins.wasm");
    const STACK_SIZE: u32 = 1_000;

    #[test]
    fn module_parse() {
        let env = Environment::new().expect("env alloc failure");
        let fib32 = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f,
            0x01, 0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x66, 0x69, 0x62, 0x00,
            0x00, 0x0a, 0x1f, 0x01, 0x1d, 0x00, 0x20, 0x00, 0x41, 0x02, 0x49, 0x04, 0x40, 0x20,
            0x00, 0x0f, 0x0b, 0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x01,
            0x6b, 0x10, 0x00, 0x6a, 0x0f, 0x0b,
        ];
        let _ = Module::parse(&env, &fib32[..]).unwrap();
    }

    #[test]
    fn test_link_functions() {
        let env = Environment::new().expect("env alloc failure");
        let runtime = Runtime::new(&env, STACK_SIZE).expect("runtime init failure");
        let mut module = runtime.parse_and_load_module(TEST_BIN).unwrap();
        module
            .link_function::<(u32, f32), f64>("env", "mul_u32_and_f32", mul_u32_and_f32_wrap)
            .unwrap();
        module
            .link_function::<(), ()>("env", "hello", hello_wrap)
            .unwrap();
    }

    #[test]
    fn test_link_closures() {
        let env = Environment::new().expect("env alloc failure");
        let runtime = Runtime::new(&env, STACK_SIZE).expect("runtime init failure");
        let mut module = runtime.parse_and_load_module(TEST_BIN).unwrap();
        module
            .link_closure(
                "env",
                "mul_u32_and_f32",
                |_ctx, args: (u32, f32)| -> TrappedResult<f64> {
                    Ok(mul_u32_and_f32(args.0, args.1))
                },
            )
            .unwrap();
        module
            .link_closure("env", "hello", |_ctx, _args: ()| -> TrappedResult<()> {
                hello()
            })
            .unwrap();
    }
}
