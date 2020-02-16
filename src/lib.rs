#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default, clippy::match_bool)]

extern crate alloc;

pub mod environment;
pub mod error;
pub mod function;
pub mod module;
pub mod runtime;

mod ty;
pub use self::ty::{WasmArg, WasmArgs, WasmType};
mod macros;
pub use self::macros::*;
mod utils;
pub(crate) mod wasm3_priv;

pub use ffi as wasm3_sys;

/// Print general wasm3 info to stdout.
#[inline]
pub fn print_m3_info() {
    unsafe { ffi::m3_PrintM3Info() };
}

/// Print wasm3 profiler info to stdout.
#[inline]
pub fn print_profiler_info() {
    unsafe { ffi::m3_PrintProfilerInfo() };
}
