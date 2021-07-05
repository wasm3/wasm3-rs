#![deny(rust_2018_idioms)]

// Required explicitly to force inclusion at link time.
#[cfg(target_env = "sgx")]
#[allow(unused_extern_crates)]
extern crate rs_libc;

use std::any::Any;
use std::boxed::Box;
use std::cell::{RefCell, UnsafeCell};
use std::convert::TryInto;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::{self, NonNull};

use impl_trait_for_tuples::impl_for_tuples;
use thiserror::Error;

/// WASM error.
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to allocate memory")]
    MemoryAllocationFailure,
    #[error("parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("module already linked")]
    ModuleAlreadyLinked,
    #[error("module too large")]
    ModuleTooLarge,
    #[error("malformed module name")]
    MalformedModuleName,
    #[error("malformed function name")]
    MalformedFunctionName,
    #[error("malformed function signature")]
    MalformedFunctionSignature,
    #[error("malformed global name")]
    MalformedGlobalName,
    #[error("global not found")]
    GlobalNotFound,
    #[error("function not found")]
    FunctionNotFound,
    #[error("function import missing")]
    FunctionImportMissing,
    #[error("argument count mismatch")]
    ArgumentCountMismatch,
    #[error("argument type mismatch")]
    ArgumentTypeMismatch,
    #[error("memory is in use")]
    MemoryInUse,
    #[error("out of memory")]
    OutOfMemory,

    // Traps.
    #[error("out of bounds memory access")]
    OutOfBoundsMemoryAccess,
    #[error("division by zero")]
    DivisionByZero,
    #[error("integer overflow")]
    IntegerOverflow,
    #[error("invalid conversion to integer")]
    InvalidIntegerConversion,
    #[error("indirect call type mismatch")]
    IndirectCallTypeMismatch,
    #[error("undefined table element")]
    UndefinedTableElement,
    #[error("null table element")]
    NullTableElement,
    #[error("program called exit")]
    ExitCalled,
    #[error("program called abort")]
    AbortCalled,
    #[error("unreachable executed")]
    UnreachableExecuted,
    #[error("stack overflow")]
    StackOverflow,

    // Other errors.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Parse error.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("incompatible WASM version")]
    IncompatibleVersion,
    #[error("malformed WASM binary")]
    Malformed,
    #[error("out of order WASM section")]
    MisorderedSection,
    #[error("underrun while parsing WASM binary")]
    Underrun,
    #[error("overrun while parsing WASM binary")]
    Overrun,
    #[error("missing init_expr in WASM binary")]
    MissingInitExpression,
    #[error("LEB-encoded value overflow")]
    LebOverflow,
    #[error("invalid length UTF-8 string")]
    MissingUtf8,
    #[error("section underrun while parsing WASM binary")]
    SectionUnderrun,
    #[error("section overrun while parsing WASM binary")]
    SectionOverrun,
    #[error("unknown value_type")]
    InvalidTypeId,
    #[error("only one memory per module is supported")]
    TooManyMemorySections,
    #[error("too many arguments or return values")]
    TooManyArgsOrRets,
}

impl Error {
    fn check_wasm3_result(err: ffi::M3Result) -> Result<(), Error> {
        unsafe {
            if err.is_null() {
                Ok(())
            } else if err == ffi::m3Err_mallocFailed {
                Err(Error::MemoryAllocationFailure)
            } else if err == ffi::m3Err_incompatibleWasmVersion {
                Err(ParseError::IncompatibleVersion.into())
            } else if err == ffi::m3Err_wasmMalformed {
                Err(ParseError::Malformed.into())
            } else if err == ffi::m3Err_misorderedWasmSection {
                Err(ParseError::MisorderedSection.into())
            } else if err == ffi::m3Err_wasmUnderrun {
                Err(ParseError::Underrun.into())
            } else if err == ffi::m3Err_wasmOverrun {
                Err(ParseError::Overrun.into())
            } else if err == ffi::m3Err_wasmMissingInitExpr {
                Err(ParseError::MissingInitExpression.into())
            } else if err == ffi::m3Err_lebOverflow {
                Err(ParseError::LebOverflow.into())
            } else if err == ffi::m3Err_missingUTF8 {
                Err(ParseError::MissingUtf8.into())
            } else if err == ffi::m3Err_wasmSectionUnderrun {
                Err(ParseError::SectionUnderrun.into())
            } else if err == ffi::m3Err_wasmSectionOverrun {
                Err(ParseError::SectionOverrun.into())
            } else if err == ffi::m3Err_invalidTypeId {
                Err(ParseError::InvalidTypeId.into())
            } else if err == ffi::m3Err_tooManyMemorySections {
                Err(ParseError::TooManyMemorySections.into())
            } else if err == ffi::m3Err_tooManyArgsRets {
                Err(ParseError::TooManyArgsOrRets.into())
            } else if err == ffi::m3Err_moduleAlreadyLinked {
                Err(Error::ModuleAlreadyLinked)
            } else if err == ffi::m3Err_functionLookupFailed {
                Err(Error::FunctionNotFound)
            } else if err == ffi::m3Err_functionImportMissing {
                Err(Error::FunctionImportMissing)
            } else if err == ffi::m3Err_malformedFunctionSignature {
                Err(Error::MalformedFunctionSignature)
            } else if err == ffi::m3Err_wasmMemoryOverflow {
                Err(Error::OutOfMemory)
            } else if err == ffi::m3Err_argumentCountMismatch {
                Err(Error::ArgumentCountMismatch)
            } else if err == ffi::m3Err_argumentTypeMismatch {
                Err(Error::ArgumentTypeMismatch)
            } else if err == ffi::m3Err_trapOutOfBoundsMemoryAccess {
                Err(Error::OutOfBoundsMemoryAccess)
            } else if err == ffi::m3Err_trapDivisionByZero {
                Err(Error::DivisionByZero)
            } else if err == ffi::m3Err_trapIntegerOverflow {
                Err(Error::IntegerOverflow)
            } else if err == ffi::m3Err_trapIntegerConversion {
                Err(Error::InvalidIntegerConversion)
            } else if err == ffi::m3Err_trapIndirectCallTypeMismatch {
                Err(Error::IndirectCallTypeMismatch)
            } else if err == ffi::m3Err_trapTableIndexOutOfRange {
                Err(Error::UndefinedTableElement)
            } else if err == ffi::m3Err_trapTableElementIsNull {
                Err(Error::NullTableElement)
            } else if err == ffi::m3Err_trapExit {
                Err(Error::ExitCalled)
            } else if err == ffi::m3Err_trapAbort {
                Err(Error::AbortCalled)
            } else if err == ffi::m3Err_trapUnreachable {
                Err(Error::UnreachableExecuted)
            } else if err == ffi::m3Err_trapStackOverflow {
                Err(Error::StackOverflow)
            } else {
                let detail = CStr::from_ptr(err).to_str().unwrap_or("???").to_string();
                Err(Error::Unknown(detail))
            }
        }
    }
}

/// Trap raised from a linked function.
#[derive(Debug)]
pub enum Trap {
    Abort,
}

impl<E: std::error::Error> From<E> for Trap {
    fn from(_error: E) -> Self {
        // Any error is converted into an abort.
        Trap::Abort
    }
}

/// WASM environment.
pub struct Environment {
    /// Underlying wasm3 environment instance.
    raw: NonNull<ffi::M3Environment>,
}

impl Environment {
    /// Create a new WASM environment.
    pub fn new() -> Result<Self, Error> {
        unsafe { NonNull::new(ffi::m3_NewEnvironment()) }
            .ok_or(Error::MemoryAllocationFailure)
            .map(|raw| Environment { raw })
    }

    /// Create a new WASM runtime within the environment.
    pub fn new_runtime<C>(
        &self,
        stack_size: u32,
        max_memory_pages: Option<u32>,
    ) -> Result<Runtime<'_, C>, Error> {
        unsafe {
            NonNull::new(ffi::m3_NewRuntime(
                self.raw.as_ptr(),
                stack_size,
                ptr::null_mut(), // We use userdata for call context, set before each call.
            ))
        }
        .ok_or(Error::MemoryAllocationFailure)
        .map(|raw| Runtime {
            raw,
            linked_functions: UnsafeCell::new(Vec::new()),
            memory_guard: RefCell::new(()),
            max_memory_pages,

            call_context_stack: UnsafeCell::new(Vec::new()),

            _env: self,
            _ctx: PhantomData,
        })
    }

    /// Parse the given module.
    pub fn parse_module<'env>(&'env self, data: &'env [u8]) -> Result<Module<'env>, Error> {
        let data_len: u32 = data.len().try_into().map_err(|_| Error::ModuleTooLarge)?;

        let mut module = ptr::null_mut();
        let result =
            unsafe { ffi::m3_ParseModule(self.raw.as_ptr(), &mut module, data.as_ptr(), data_len) };
        Error::check_wasm3_result(result)?;

        let raw = NonNull::new(module).expect("module must not be null on successful return");
        Ok(Module {
            raw: Some(raw),
            _env: self,
        })
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeEnvironment(self.raw.as_ptr()) };
    }
}

/// Data stored within the userdata of a context for the duration of a call.
struct RuntimeUserdata<'env, 'rt, C> {
    /// A pointer to the instance that initiated the call.
    instance: *const Instance<'env, 'rt, C>,
}

/// WASM runtime.
pub struct Runtime<'env, C> {
    /// Underlying wasm3 runtime instance.
    raw: NonNull<ffi::M3Runtime>,
    /// Functions linked into the runtime.
    linked_functions: UnsafeCell<Vec<Pin<Box<dyn Any + 'static>>>>,
    /// Guard for access to runtime's memory.
    memory_guard: RefCell<()>,
    /// Maximum amount of memory pages that can be allocated by the guest.
    max_memory_pages: Option<u32>,

    /// A stack of call context pointers, one for each (sub)call.
    ///
    /// Note that the contained pointer actually points to C.
    call_context_stack: UnsafeCell<Vec<*mut ()>>,

    _env: &'env Environment,
    _ctx: PhantomData<C>,
}

impl<'env, C> Runtime<'env, C> {
    /// Loads a parsed module into this runtime.
    pub fn load_module<'rt>(
        &'rt self,
        mut module: Module<'_>,
    ) -> Result<Instance<'env, 'rt, C>, Error> {
        let raw_module = module.raw.take().expect("module has not yet been loaded");
        let result = unsafe { ffi::m3_LoadModule(self.raw.as_ptr(), raw_module.as_ptr()) };
        Error::check_wasm3_result(result)?;

        // HACK: There is no proper API for setting memory limits and every time a module is loaded
        //       the memory limit is updated so we need to set it back in case we enforce limits.
        if let Some(max_memory_pages) = self.max_memory_pages {
            unsafe {
                (*self.raw.as_ptr()).memory.maxPages = max_memory_pages;
            }
        }

        Ok(Instance {
            raw: raw_module,
            rt: self,
        })
    }

    /// Runs the given function with safe access to raw WASM memory.
    ///
    /// In case memory is already in use this function will return `Error::MemoryInUse`.
    pub fn try_with_memory<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Memory<'_>) -> R,
    {
        // Get exclusive access to the memory instance.
        let _guard = self
            .memory_guard
            .try_borrow_mut()
            .map_err(|_| Error::MemoryInUse)?;

        unsafe {
            // This is safe as memory can only be accessed while this function is live and since
            // we hold a mutable reference to the guard, this ensures that no function calls can
            // happen while we're running and thus no reallocations that would invalidate the
            // pointer.
            let memory = Memory::from_runtime(self.raw.as_ref());
            Ok(f(memory))
        }
    }
}

impl<'env, C> Drop for Runtime<'env, C> {
    fn drop(&mut self) {
        unsafe { ffi::m3_FreeRuntime(self.raw.as_ptr()) }
    }
}

/// WASM memory.
pub struct Memory<'m> {
    raw: &'m mut [u8],
}

impl<'m> Memory<'m> {
    /// Memory size.
    pub fn size(&self) -> usize {
        self.raw.len()
    }

    /// Returns the memory as a slice.
    pub fn as_slice(&self) -> &[u8] {
        self.raw
    }

    /// Returns the memory as a mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        self.raw
    }

    /// Create a WASM memory reference for the passed runtime.
    ///
    /// # Safety
    ///
    /// Using the returned memory is unsafe because the pointer may be invalidated by any function
    /// calls due to reallocations.
    unsafe fn from_runtime<'a>(rt: *const ffi::M3Runtime) -> Memory<'a> {
        let header = (*rt).memory.mallocated;
        let len = (*header).length as usize;
        let data = if len == 0 {
            NonNull::dangling().as_ptr()
        } else {
            header.offset(1).cast()
        };

        Memory {
            raw: std::slice::from_raw_parts_mut(data, len),
        }
    }
}

/// A parsed WASM module, ready to be loaded into a runtime.
pub struct Module<'env> {
    raw: Option<NonNull<ffi::M3Module>>,
    _env: &'env Environment,
}

impl<'env> Drop for Module<'env> {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            unsafe { ffi::m3_FreeModule(raw.as_ptr()) }
        }
    }
}

/// Linked function call context.
pub struct CallContext<'call, C> {
    /// Runtime instance.
    pub instance: &'call Instance<'call, 'call, C>,
    /// Externally provided per-call context (optional).
    pub context: Option<&'call mut C>,
}

/// WASM module instance loaded into a runtime.
pub struct Instance<'env, 'rt, C> {
    raw: NonNull<ffi::M3Module>,
    rt: &'rt Runtime<'env, C>,
}

impl<'env, 'rt, C> Instance<'env, 'rt, C> {
    /// Reference to the runtime hosting this instance.
    pub fn runtime(&self) -> &'rt Runtime<'env, C> {
        self.rt
    }

    /// Finds a function with the given name and argument/return types.
    pub fn find_function<A, R>(&self, name: &str) -> Result<Function<'_, '_, '_, C, A, R>, Error>
    where
        A: Arg,
        R: Arg,
    {
        let name = CString::new(name).map_err(|_| Error::MalformedFunctionName)?;
        let mut func = ptr::null_mut();
        // TODO: Limit scope to just this module once we have https://github.com/wasm3/wasm3/issues/267.
        let result =
            unsafe { ffi::m3_FindFunction(&mut func, self.rt.raw.as_ptr(), name.as_ptr()) };
        Error::check_wasm3_result(result)?;

        let raw = NonNull::new(func).expect("function must not be null on successful return");
        Function::new(self, raw)
    }

    /// Links an imported function.
    pub fn link_function<F, A, R>(
        &mut self,
        module_name: &str,
        function_name: &str,
        f: F,
    ) -> Result<(), Error>
    where
        F: FnMut(CallContext<'_, C>, A) -> Result<R, Trap> + 'static,
        A: Arg,
        R: Arg,
    {
        unsafe extern "C" fn _impl<C, F, A, R>(
            runtime: ffi::IM3Runtime,
            ctx: ffi::IM3ImportContext,
            sp: *mut u64,
            _mem: *mut cty::c_void,
        ) -> *const cty::c_void
        where
            F: FnMut(CallContext<'_, C>, A) -> Result<R, Trap> + 'static,
            A: Arg,
            R: Arg,
        {
            // The stack layout is as follows:
            //
            // <return-value-0>
            // ...
            // <return-value-n>
            // <arg-0>
            // ...
            // <arg-m>

            let num_args = A::count();
            let num_rets = R::count();
            let sp = std::slice::from_raw_parts_mut(sp, num_args + num_rets);
            // First argument starts at the num_rets offset.
            let args = match A::from_stack(&sp[num_rets..]) {
                Ok(args) => args,
                Err(_err) => return ffi::m3Err_trapAbort as _,
            };

            // Create function call context. We first use the userdata pointer to get the
            // RuntimeUserdata structure which contains a pointer to the Runtime. From the Runtime
            // we get the current call context from the call context stack.
            let userdata = &*(*runtime).userdata.cast::<RuntimeUserdata<'_, '_, C>>();
            let instance = &*userdata.instance;
            let cc_stack = &*instance.rt.call_context_stack.get();
            let context = *cc_stack.last().unwrap();
            let context = if context.is_null() {
                // Context may be null in case a regular call without a context was used.
                None
            } else {
                Some(&mut *context.cast::<C>())
            };
            let context = CallContext { instance, context };

            // Execute function.
            let userdata = (*ctx).userdata;
            let ret = match (&mut *userdata.cast::<F>())(context, args) {
                Ok(ret) => ret,
                Err(Trap::Abort) => return ffi::m3Err_trapAbort as _,
            };

            // Put results on stack.
            match ret.to_stack(&mut sp[0..num_rets]) {
                Ok(()) => ffi::m3Err_none as _,
                Err(_err) => ffi::m3Err_trapAbort as _,
            }
        }

        let module_name = CString::new(module_name).map_err(|_| Error::MalformedModuleName)?;
        let function_name =
            CString::new(function_name).map_err(|_| Error::MalformedFunctionName)?;

        // Generate type signature.
        let mut signature = String::new();
        R::push_signature(&mut signature);
        signature.push('(');
        A::push_signature(&mut signature);
        signature.push(')');
        let signature = CString::new(signature).unwrap();

        // Pin closure to prevent it being moved.
        let mut f = Box::pin(f);

        let result = unsafe {
            ffi::m3_LinkRawFunctionEx(
                self.raw.as_ptr(),
                module_name.as_ptr(),
                function_name.as_ptr(),
                signature.as_ptr(),
                Some(_impl::<C, F, A, R>),
                (f.as_mut().get_unchecked_mut() as *mut F).cast(),
            )
        };
        Error::check_wasm3_result(result)?;

        // Save closure so it stays put until the runtime is dropped. This is safe as none of these
        // types can be used from multiple threads.
        unsafe {
            let linked_functions = &mut *self.rt.linked_functions.get();
            linked_functions.push(f);
        }

        Ok(())
    }

    /// Return the value of the given exported global.
    pub fn get_global<T: Arg>(&self, name: &str) -> Result<T, Error> {
        let global_name = CString::new(name).map_err(|_| Error::MalformedGlobalName)?;
        let global = unsafe { ffi::m3_FindGlobal(self.raw.as_ptr(), global_name.as_ptr()) };
        if global.is_null() {
            return Err(Error::GlobalNotFound);
        }

        let mut value = ffi::M3TaggedValue {
            type_: ffi::M3ValueType::c_m3Type_unknown,
            value: ffi::M3TaggedValue_M3ValueUnion { i64_: 0 },
        };
        let result = unsafe { ffi::m3_GetGlobal(global, &mut value) };
        Error::check_wasm3_result(result)?;

        Arg::from_tagged(value)
    }

    /// Set the value of the given exported global.
    pub fn set_global<T: Arg>(&self, name: &str, value: T) -> Result<(), Error> {
        let global_name = CString::new(name).map_err(|_| Error::MalformedGlobalName)?;
        let global = unsafe { ffi::m3_FindGlobal(self.raw.as_ptr(), global_name.as_ptr()) };
        if global.is_null() {
            return Err(Error::GlobalNotFound);
        }

        let mut value = value.to_tagged()?;
        let result = unsafe { ffi::m3_SetGlobal(global, &mut value) };
        Error::check_wasm3_result(result)?;

        Ok(())
    }

    /// Executes given closure with a call context set.
    fn with_call_context<F, R>(&self, ctx: Option<&mut C>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let mut userdata = RuntimeUserdata {
            instance: self as *const Instance<'_, '_, C>,
        };

        struct CallContextGuard<'a, 'env, 'rt, C> {
            rt: &'rt Runtime<'env, C>,
            userdata: &'a mut RuntimeUserdata<'env, 'rt, C>,
        }

        impl<'a, 'env, 'rt, C> CallContextGuard<'a, 'env, 'rt, C> {
            fn new(
                rt: &'rt Runtime<'env, C>,
                userdata: &'a mut RuntimeUserdata<'env, 'rt, C>,
                ctx: Option<&mut C>,
            ) -> Self {
                let rt_ptr = rt.raw.as_ptr();
                unsafe {
                    // Obtain mutable reference to the CC stack. Since this code is single-threaded
                    // this is safe as we are only using the reference in this block.
                    let cc_stack = &mut *rt.call_context_stack.get();
                    assert!(cc_stack.is_empty() == (*rt_ptr).userdata.is_null());

                    // Make sure that userdata points to the current instance. Note that accessing
                    // it is safe as long as F is running as we are holding a reference to  it (in
                    // the form of &self).
                    if cc_stack.is_empty() {
                        // NOTE: Even though userdata is a mutable reference, we never use it for
                        //       mutating any data in it, only for accessing it. This is only
                        //       required because the FFI bindings require a *mut c_void.
                        (*rt_ptr).userdata = (userdata as *mut RuntimeUserdata<'_, '_, C>).cast();
                    }

                    // Push call context to stack.
                    cc_stack.push(
                        ctx.map(|ctx| (ctx as *mut C).cast())
                            .unwrap_or(ptr::null_mut()),
                    );
                }

                CallContextGuard { rt, userdata }
            }
        }

        impl<'a, 'env, 'rt, C> Drop for CallContextGuard<'a, 'env, 'rt, C> {
            fn drop(&mut self) {
                let rt_ptr = self.rt.raw.as_ptr();
                unsafe {
                    // Obtain mutable reference to the CC stack. Since this code is single-threaded
                    // this is safe as we are only using the reference in this block.
                    let cc_stack = &mut *self.rt.call_context_stack.get();
                    assert!(!cc_stack.is_empty() && !(*rt_ptr).userdata.is_null());

                    // Pop call context from stack.
                    cc_stack.pop();

                    // If call context stack is empty, reset userdata as it may no longer be valid.
                    if cc_stack.is_empty() {
                        assert!(
                            (*rt_ptr).userdata
                                == (self.userdata as *mut RuntimeUserdata<'_, '_, C>).cast()
                        );
                        (*rt_ptr).userdata = ptr::null_mut();
                    }
                }
            }
        }

        // Create guard that will manage the call context stack.
        let _guard = CallContextGuard::new(self.rt, &mut userdata, ctx);

        f()
    }
}

/// Trait for types which can be passed as WASM function arguments.
pub trait Arg {
    /// Push internal interpeter types for this argument.
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>);

    /// Push signature for this argument.
    fn push_signature(sig: &mut String);

    /// Push pointers to argument values for this argument.
    fn push_values(&self, values: &mut Vec<*const cty::c_void>);

    /// Read argument from stack.
    fn from_stack(sp: &[u64]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Write argument to stack.
    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error>;

    /// Decode value from tagged.
    fn from_tagged(_value: ffi::M3TaggedValue) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Err(Error::ArgumentTypeMismatch)
    }

    // Encode value to tagged.
    fn to_tagged(&self) -> Result<ffi::M3TaggedValue, Error> {
        Err(Error::ArgumentTypeMismatch)
    }

    /// The number of arguments.
    fn count() -> usize {
        1
    }

    /// Create a default instance of the argument.
    fn default() -> Self;
}

#[impl_for_tuples(30)]
impl Arg for Tuple {
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>) {
        for_tuples!( #( Tuple::push_types(types); )* );
    }

    fn push_signature(sig: &mut String) {
        for_tuples!( #( Tuple::push_signature(sig); )* );
    }

    fn push_values(&self, values: &mut Vec<*const cty::c_void>) {
        for_tuples!( #( Tuple.push_values(values); )* );
    }

    #[allow(clippy::eval_order_dependence)]
    fn from_stack(mut sp: &[u64]) -> Result<Self, Error> {
        let mut index = 0;

        let result = for_tuples!( ( #( {
            if sp.len() < index + 1 {
                return Err(Error::ArgumentCountMismatch);
            }

            let item = Tuple::from_stack( &sp[index..] )?;
            index += Tuple::count();

            item
        } ),* ) );

        Ok(result)
    }

    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error> {
        let mut index = 0;

        for_tuples!( #( {
            if sp.len() < index + 1 {
                return Err(Error::ArgumentCountMismatch);
            }

            Tuple.to_stack(&mut sp[index..])?;
            index += Tuple::count();
        } )* );

        Ok(())
    }

    #[allow(clippy::let_and_return)]
    fn count() -> usize {
        let mut result = 0;
        for_tuples!( #( result += Tuple::count(); )* );
        result
    }

    #[allow(clippy::unused_unit)]
    fn default() -> Self {
        for_tuples!( ( #( Tuple::default() ),* ) )
    }
}

impl Arg for u64 {
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>) {
        types.push(ffi::M3ValueType::c_m3Type_i64);
    }

    fn push_signature(sig: &mut String) {
        sig.push('I');
    }

    fn push_values(&self, args: &mut Vec<*const cty::c_void>) {
        args.push((self as *const u64).cast());
    }

    fn from_stack(sp: &[u64]) -> Result<Self, Error> {
        Ok(sp[0])
    }

    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error> {
        sp[0] = *self;
        Ok(())
    }

    fn from_tagged(value: ffi::M3TaggedValue) -> Result<Self, Error> {
        if value.type_ == ffi::M3ValueType::c_m3Type_i64 {
            unsafe { Ok(value.value.i64_) }
        } else {
            Err(Error::ArgumentTypeMismatch)
        }
    }

    fn to_tagged(&self) -> Result<ffi::M3TaggedValue, Error> {
        Ok(ffi::M3TaggedValue {
            type_: ffi::M3ValueType::c_m3Type_i64,
            value: ffi::M3TaggedValue_M3ValueUnion { i64_: *self },
        })
    }

    fn default() -> Self {
        0
    }
}

impl Arg for i64 {
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>) {
        types.push(ffi::M3ValueType::c_m3Type_i64);
    }

    fn push_signature(sig: &mut String) {
        sig.push('I');
    }

    fn push_values(&self, args: &mut Vec<*const cty::c_void>) {
        args.push((self as *const i64).cast());
    }

    fn from_stack(sp: &[u64]) -> Result<Self, Error> {
        Ok(sp[0] as i64)
    }

    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error> {
        sp[0] = *self as u64;
        Ok(())
    }

    fn from_tagged(value: ffi::M3TaggedValue) -> Result<Self, Error> {
        if value.type_ == ffi::M3ValueType::c_m3Type_i64 {
            unsafe { Ok(value.value.i64_ as i64) }
        } else {
            Err(Error::ArgumentTypeMismatch)
        }
    }

    fn to_tagged(&self) -> Result<ffi::M3TaggedValue, Error> {
        Ok(ffi::M3TaggedValue {
            type_: ffi::M3ValueType::c_m3Type_i64,
            value: ffi::M3TaggedValue_M3ValueUnion { i64_: *self as u64 },
        })
    }

    fn default() -> Self {
        0
    }
}

impl Arg for u32 {
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>) {
        types.push(ffi::M3ValueType::c_m3Type_i32);
    }

    fn push_signature(sig: &mut String) {
        sig.push('i');
    }

    fn push_values(&self, args: &mut Vec<*const cty::c_void>) {
        args.push((self as *const u32).cast());
    }

    fn from_stack(sp: &[u64]) -> Result<Self, Error> {
        Ok(sp[0] as u32)
    }

    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error> {
        sp[0] = *self as u64;
        Ok(())
    }

    fn from_tagged(value: ffi::M3TaggedValue) -> Result<Self, Error> {
        if value.type_ == ffi::M3ValueType::c_m3Type_i32 {
            unsafe { Ok(value.value.i32_) }
        } else {
            Err(Error::ArgumentTypeMismatch)
        }
    }

    fn to_tagged(&self) -> Result<ffi::M3TaggedValue, Error> {
        Ok(ffi::M3TaggedValue {
            type_: ffi::M3ValueType::c_m3Type_i32,
            value: ffi::M3TaggedValue_M3ValueUnion { i32_: *self },
        })
    }

    fn default() -> Self {
        0
    }
}

impl Arg for i32 {
    fn push_types(types: &mut Vec<ffi::M3ValueType::Type>) {
        types.push(ffi::M3ValueType::c_m3Type_i32);
    }

    fn push_signature(sig: &mut String) {
        sig.push('i');
    }

    fn push_values(&self, args: &mut Vec<*const cty::c_void>) {
        args.push((self as *const i32).cast());
    }

    fn from_stack(sp: &[u64]) -> Result<Self, Error> {
        Ok(sp[0] as i32)
    }

    fn to_stack(&self, sp: &mut [u64]) -> Result<(), Error> {
        sp[0] = *self as u64;
        Ok(())
    }

    fn from_tagged(value: ffi::M3TaggedValue) -> Result<Self, Error> {
        if value.type_ == ffi::M3ValueType::c_m3Type_i32 {
            unsafe { Ok(value.value.i32_ as i32) }
        } else {
            Err(Error::ArgumentTypeMismatch)
        }
    }

    fn to_tagged(&self) -> Result<ffi::M3TaggedValue, Error> {
        Ok(ffi::M3TaggedValue {
            type_: ffi::M3ValueType::c_m3Type_i32,
            value: ffi::M3TaggedValue_M3ValueUnion { i32_: *self as u32 },
        })
    }

    fn default() -> Self {
        0
    }
}

/// WASM function.
pub struct Function<'fun, 'env, 'rt, C, A: Arg, R: Arg> {
    raw: NonNull<ffi::M3Function>,
    instance: &'fun Instance<'env, 'rt, C>,

    _args: PhantomData<A>,
    _result: PhantomData<R>,
}

impl<'fun, 'env, 'rt, C, A: Arg, R: Arg> Function<'fun, 'env, 'rt, C, A, R> {
    fn new(
        instance: &'fun Instance<'env, 'rt, C>,
        raw: NonNull<ffi::M3Function>,
    ) -> Result<Self, Error> {
        // Validate argument types.
        let mut types = vec![];
        A::push_types(&mut types);

        let arg_count = unsafe { ffi::m3_GetArgCount(raw.as_ptr()) };
        if types.len() != arg_count as usize {
            return Err(Error::ArgumentCountMismatch);
        }

        for index in 0..arg_count {
            let arg_type = unsafe { ffi::m3_GetArgType(raw.as_ptr(), index) };
            if arg_type != types[index as usize] {
                return Err(Error::ArgumentTypeMismatch);
            }
        }

        // Validate return argument types.
        let mut types = vec![];
        R::push_types(&mut types);

        let arg_count = unsafe { ffi::m3_GetRetCount(raw.as_ptr()) };
        if types.len() != arg_count as usize {
            return Err(Error::ArgumentCountMismatch);
        }

        for index in 0..arg_count {
            let arg_type = unsafe { ffi::m3_GetRetType(raw.as_ptr(), index) };
            if arg_type != types[index as usize] {
                return Err(Error::ArgumentTypeMismatch);
            }
        }

        Ok(Function {
            raw,
            instance,
            _args: PhantomData,
            _result: PhantomData,
        })
    }

    fn raw_call(&self, args: A) -> Result<R, Error> {
        // Make sure that we are not being called within a `try_with_memory` block as that could
        // cause those memory references to be invalidated.
        let guard = self
            .instance
            .runtime()
            .memory_guard
            .try_borrow()
            .map_err(|_| Error::MemoryInUse)?;
        // Drop the guard immediately as `try_with_memory` is allowed to be called from within
        // linked functions.
        drop(guard);

        // Prepare arguments for the call.
        let mut raw_args = vec![];
        args.push_values(&mut raw_args);

        let result = unsafe {
            ffi::m3_Call(
                self.raw.as_ptr(),
                raw_args.len() as u32,
                raw_args.as_mut_ptr(),
            )
        };
        Error::check_wasm3_result(result)?;

        // Prepare return value placeholders for the call.
        let rets = R::default();
        let mut raw_rets = vec![];
        rets.push_values(&mut raw_rets);

        let result = unsafe {
            ffi::m3_GetResults(
                self.raw.as_ptr(),
                raw_rets.len() as u32,
                raw_rets.as_mut_ptr(),
            )
        };
        Error::check_wasm3_result(result)?;

        Ok(rets)
    }

    /// Call a function with the given arguments.
    pub fn call(&self, args: A) -> Result<R, Error> {
        self.instance
            .with_call_context(None, || self.raw_call(args))
    }

    /// Call a function with the given arguments and the provided call context.
    pub fn call_with_context(&self, ctx: &mut C, args: A) -> Result<R, Error> {
        self.instance
            .with_call_context(Some(ctx), || self.raw_call(args))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const FIB32_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01,
        0x7f, 0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x66, 0x69, 0x62, 0x00, 0x00, 0x0a,
        0x1f, 0x01, 0x1d, 0x00, 0x20, 0x00, 0x41, 0x02, 0x49, 0x04, 0x40, 0x20, 0x00, 0x0f, 0x0b,
        0x20, 0x00, 0x41, 0x02, 0x6b, 0x10, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6b, 0x10, 0x00, 0x6a,
        0x0f, 0x0b,
    ];

    #[test]
    fn test_parse_module() {
        let env = Environment::new().unwrap();
        let _ = env.parse_module(FIB32_WASM).unwrap();
    }

    #[test]
    fn test_call_function() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env.parse_module(FIB32_WASM).unwrap();
        let instance = rt.load_module(module).unwrap();

        let func = instance.find_function::<i32, i32>("fib").unwrap();
        let result = func.call(10).unwrap();
        assert_eq!(result, 55);
    }

    #[test]
    fn test_link_function() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env
            .parse_module(
                &include_bytes!("../tests/wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")
                    [..],
            )
            .unwrap();
        let mut instance = rt.load_module(module).unwrap();

        const MILLIS: u64 = 10_000; // This value will be converted to seconds.
        instance
            .link_function("time", "millis", |_, ()| Ok(MILLIS))
            .unwrap();

        let func = instance.find_function::<(), u64>("seconds").unwrap();
        let result = func.call(()).unwrap();
        assert_eq!(result, MILLIS / 1000);
    }

    #[test]
    fn test_link_function_trap() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env
            .parse_module(
                &include_bytes!("../tests/wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")
                    [..],
            )
            .unwrap();
        let mut instance = rt.load_module(module).unwrap();

        instance
            .link_function("time", "millis", |_, ()| -> Result<u64, Trap> {
                Err(Trap::Abort)
            })
            .unwrap();

        let func = instance.find_function::<(), u64>("seconds").unwrap();
        func.call(()).expect_err("call should abort with trap");
    }

    #[test]
    fn test_link_function_with_context() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<u16>(60 * 1024, Some(16)).unwrap();
        let module = env
            .parse_module(
                &include_bytes!("../tests/wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")
                    [..],
            )
            .unwrap();
        let mut instance = rt.load_module(module).unwrap();

        const MILLIS: u64 = 10_000; // This value will be converted to seconds.
        instance
            .link_function("time", "millis", |ctx, ()| {
                *ctx.context.unwrap() = 42;
                Ok(MILLIS)
            })
            .unwrap();

        let mut call_context = 0;
        let func = instance.find_function::<(), u64>("seconds").unwrap();
        let result = func.call_with_context(&mut call_context, ()).unwrap();
        assert_eq!(result, MILLIS / 1000);

        assert_eq!(
            call_context, 42,
            "context should be updated from linked function"
        );
    }

    #[test]
    fn test_nested_with_call_context() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env.parse_module(FIB32_WASM).unwrap();
        let instance = rt.load_module(module).unwrap();
        instance.with_call_context(Some(&mut ()), || {
            instance.with_call_context(Some(&mut ()), || {})
        });
    }

    #[test]
    fn test_try_with_memory() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env
            .parse_module(
                &include_bytes!("../tests/wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")
                    [..],
            )
            .unwrap();
        let mut instance = rt.load_module(module).unwrap();

        const MILLIS: u64 = 10_000; // This value will be converted to seconds.
        const OFFSET: usize = 42;
        const EXPECTED_VALUE: u8 = 0xBA;
        instance
            .link_function("time", "millis", |ctx, ()| {
                // Memory access from within linked functions should work.
                ctx.instance
                    .runtime()
                    .try_with_memory(|mut memory| {
                        memory.as_slice_mut()[OFFSET] = EXPECTED_VALUE; // Modify memory location.
                    })
                    .unwrap();

                Ok(MILLIS)
            })
            .unwrap();

        let func = instance.find_function::<(), u64>("seconds").unwrap();
        let result = func.call(()).unwrap();
        assert_eq!(result, MILLIS / 1000);

        let value = rt
            .try_with_memory(|memory| memory.as_slice()[OFFSET])
            .unwrap();
        assert_eq!(value, EXPECTED_VALUE);
    }

    #[test]
    fn test_try_with_memory_call_fail() {
        let env = Environment::new().unwrap();
        let rt = env.new_runtime::<()>(60 * 1024, Some(16)).unwrap();
        let module = env.parse_module(FIB32_WASM).unwrap();
        let instance = rt.load_module(module).unwrap();

        rt.try_with_memory(|_memory| {
            // Calling a function while having a reference to a memory should fail.
            let func = instance.find_function::<i32, i32>("fib").unwrap();
            func.call(10)
                .expect_err("calling functions within `try_with_memory` block should fail")
        })
        .unwrap();
    }
}
