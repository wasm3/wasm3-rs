#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
//! A rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

extern crate alloc;

pub mod error;

mod environment;
pub use self::environment::Environment;
mod function;
pub use self::function::{CallContext, Function, RawCall};
mod macros;
pub use self::macros::*;
mod module;
pub use self::module::{Module, ParsedModule};
mod runtime;
pub use self::runtime::Runtime;
mod ty;
pub use self::ty::{WasmArg, WasmArgs, WasmType};
mod utils;
pub use ffi as wasm3_sys;
