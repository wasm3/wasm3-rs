#![no_std]

#[link(wasm_import_module = "time")]
extern "C" {
    #[link_name = "millis"]
    fn millis() -> u64;
}

#[no_mangle]
pub extern "C" fn seconds() -> u64 {
    unsafe { millis() / 1000 }
}

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    unreachable!()
}
