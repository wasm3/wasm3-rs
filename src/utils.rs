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
    core::slice::from_raw_parts(start, len)
}

pub unsafe fn cstr_to_str<'a>(ptr: *const libc::c_char) -> &'a str {
    core::str::from_utf8_unchecked(bytes_till_null(ptr))
}

pub unsafe fn eq_cstr_str(cstr: *const libc::c_char, str: &str) -> bool {
    if cstr.is_null() {
        return false;
    }
    let mut bytes = str.as_bytes().iter();
    let mut cstr = cstr.cast::<u8>();
    loop {
        match (bytes.next(), *cstr) {
            (None, 0) => break true,
            (Some(_), 0) => break false,
            (Some(&byte), cbyte) if cbyte == byte => cstr = cstr.add(1),
            _ => break false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_till_null() {
        let bytes_null = b"abcdef\0";
        let bytes = b"abcdef";
        assert_eq!(
            unsafe { bytes_till_null(bytes_null.as_ptr().cast()) },
            bytes
        );
    }

    #[test]
    fn test_bytes_till_null_null() {
        assert_eq!(unsafe { bytes_till_null(core::ptr::null()) }, &[]);
    }

    #[test]
    fn test_cstr_to_str() {
        let cstr = b"abcdef\0";
        let str = "abcdef";
        assert_eq!(unsafe { cstr_to_str(cstr.as_ptr().cast()) }, str);
    }

    #[test]
    fn test_eq_cstr_str_is_eq() {
        let cstr = b"abcdef\0";
        let str = "abcdef";
        assert!(unsafe { eq_cstr_str(cstr.as_ptr().cast(), str) });
    }

    #[test]
    fn test_eq_cstr_str_is_null() {
        assert!(unsafe { !eq_cstr_str(core::ptr::null(), "") });
    }

    #[test]
    fn test_eq_cstr_str_is_neq() {
        let cstr = b"abcdef\0";
        let str = "abcgef";
        assert!(unsafe { !eq_cstr_str(cstr.as_ptr().cast(), str) });
    }

    #[test]
    fn test_eq_cstr_str_is_neq_null() {
        let cstr = b"abcdef\0";
        let str = "abcdef\0";
        assert!(unsafe { !eq_cstr_str(cstr.as_ptr().cast(), str) });
    }

    #[test]
    fn test_eq_cstr_str_is_neq_beyond() {
        let cstr = b"abcdef\0";
        let str = "abcdef\0agsdg";
        assert!(unsafe { !eq_cstr_str(cstr.as_ptr().cast(), str) });
    }

    #[test]
    fn test_eq_cstr_str_is_neq_short() {
        let cstr = b"abcdef\0";
        let str = "abc";
        assert!(unsafe { !eq_cstr_str(cstr.as_ptr().cast(), str) });
    }
}
