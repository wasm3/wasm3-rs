#![no_std]
#![warn(missing_docs)]
//! A rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

extern crate alloc;

mod environment;
pub mod error;
mod function;
mod macros;
mod module;
mod runtime;
mod ty;
mod utils;

pub use self::environment::Environment;
pub use self::function::{CallContext, Function, RawCall};
pub use self::module::{Module, ParsedModule};
pub use self::runtime::Runtime;
pub use self::ty::{WasmArg, WasmArgs, WasmType};
pub use ffi as wasm3_sys;
