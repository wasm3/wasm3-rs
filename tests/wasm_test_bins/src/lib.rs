#![no_std]

extern "C" {
    fn mul_u32_and_f32(a: u32, b: f32) -> f64;
    fn hello();
}

#[no_mangle]
pub extern "C" fn add_u64(foo: u64, bar: u64) -> u64 {
    foo + bar
}

#[no_mangle]
pub extern "C" fn add_u32(foo: u32, bar: u32) -> u32 {
    foo + bar
}

#[no_mangle]
pub extern "C" fn invert(foo: u64) -> u64 {
    !foo
}

#[no_mangle]
pub extern "C" fn constant() -> u64 {
    0xDEAD_BEEF_0000_FFFF
}

#[no_mangle]
pub extern "C" fn no_return(_foo: u64) {}

#[no_mangle]
pub extern "C" fn u32() -> u32 {
    0xDEAD_BEEF
}

#[no_mangle]
pub extern "C" fn empty() {}

#[no_mangle]
pub extern "C" fn memory_size() -> u32 {
    core::arch::wasm32::memory_size(0) as u32
}

#[no_mangle]
pub extern "C" fn call_imports() {
    unsafe {
        hello();
        let _ = mul_u32_and_f32(1, 1.0);
    }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    unreachable!()
}
