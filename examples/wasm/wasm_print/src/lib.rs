#![no_std]

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let stdout = 1;
    let message = "Hello, World!\n";
    let data = [wasi::Ciovec {
        buf: message.as_ptr(),
        buf_len: message.len(),
    }];
    wasi::fd_write(stdout, &data).unwrap();
}
