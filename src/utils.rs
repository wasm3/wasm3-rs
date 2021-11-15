use alloc::vec::Vec;

pub unsafe fn bytes_till_null<'a>(ptr: *const cty::c_char) -> &'a [u8] {
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

pub unsafe fn cstr_to_str<'a>(ptr: *const cty::c_char) -> &'a str {
    core::str::from_utf8_unchecked(bytes_till_null(ptr))
}

pub fn str_to_cstr_owned(str: &str) -> Vec<cty::c_char> {
    let mut cstr = Vec::with_capacity(str.as_bytes().len() + 1);
    cstr.extend(str.bytes().map(|c| c as cty::c_char));
    cstr.push(0 as cty::c_char);
    cstr
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
    fn test_str_to_cstr_owned_is_null_terminated() {
        let str = "abcdef";
        let cstr = b"abcdef\0";
        assert_eq!(str_to_cstr_owned(str).as_slice(), &cstr.map(|c| c as i8));
    }
}
