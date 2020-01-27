#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all)]
pub mod environment;
pub mod error;
pub mod function;
pub mod module;
pub mod runtime;
mod ty;
pub use self::ty::{WasmArg, WasmArgs, WasmType};
mod macros;
pub use self::macros::*;

#[inline]
pub fn print_m3_info() {
    unsafe { ffi::m3_PrintM3Info() };
}

#[inline]
pub fn print_profiler_info() {
    unsafe { ffi::m3_PrintProfilerInfo() };
}

pub(crate) unsafe fn bytes_till_null<'a>(ptr: *const libc::c_char) -> &'a [u8] {
    let start = ptr.cast::<u8>();
    let mut ptr = start;
    let mut len = 0;
    while *ptr != 0 {
        ptr = ptr.add(1);
        len += 1;
    }
    core::slice::from_raw_parts(start, len - 1)
}
