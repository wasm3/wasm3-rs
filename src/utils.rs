pub unsafe fn bytes_till_null<'a>(ptr: *const libc::c_char) -> &'a [u8] {
    if ptr.is_null() {
        return &[];
    }
    let start = ptr.cast::<u8>();
    let mut ptr = start;
    let mut len = 0;
    while *ptr != 0 {
        ptr = ptr.add(1);
        len += 1;
    }
    core::slice::from_raw_parts(start, len - 1)
}

pub unsafe fn cstr_to_str<'a>(ptr: *const libc::c_char) -> &'a str {
    core::str::from_utf8_unchecked(bytes_till_null(ptr))
}

pub fn eq_cstr_str(cstr: *const libc::c_char, str: &str) -> bool {
    if cstr.is_null() {
        return false;
    }
    let mut bytes = str.as_bytes().iter();
    let mut cstr = cstr.cast::<u8>();
    loop {
        match (bytes.next(), unsafe { *cstr }) {
            (None, 0) => break true,
            (Some(_), 0) => break false,
            (Some(&byte), cbyte) if cbyte == byte => unsafe {
                cstr = cstr.add(1);
            },
            _ => break false,
        }
    }
}
