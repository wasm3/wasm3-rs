/// Exposes some of wasm3's private api required for this crate to work properly.
use ffi::*;

extern "C" {
    pub fn op_CallRawFunction(
        _pc: pc_t,
        _sp: *mut u64,
        _mem: *mut M3MemoryHeader,
        _r0: m3reg_t,
        _fp0: f64,
    ) -> m3ret_t;
    pub fn op_CallRawFunctionEx(
        _pc: pc_t,
        _sp: *mut u64,
        _mem: *mut M3MemoryHeader,
        _r0: m3reg_t,
        _fp0: f64,
    ) -> m3ret_t;
    pub fn EmitWord_impl(i_page: IM3CodePage, i_word: *mut cty::c_void);
    pub fn Compile_Function(io_function: IM3Function) -> M3Result;
    pub fn AcquireCodePageWithCapacity(io_runtime: IM3Runtime, i_slotCount: u32) -> IM3CodePage;
    pub fn ReleaseCodePage(io_runtime: IM3Runtime, i_codePage: IM3CodePage);
    pub fn GetPagePC(i_page: IM3CodePage) -> pc_t;
}
