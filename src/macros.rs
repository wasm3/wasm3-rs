//! Public macros

#[macro_export]
macro_rules! make_func_wrapper {
    // ptype is an ident because we still want to match on it later -- \/                  rtype too -- \/
    ( $wis:vis $wrapper_name:ident: $original:ident( $( $pname:ident: $ptype:ident ),* $( , )? ) $( -> $rtype:ident )?) => {
        $wis unsafe extern "C" fn $wrapper_name(
            _rt: ffi::IM3Runtime,
            _sp: *mut u64,
            _mem: *mut core::ffi::c_void,
        ) -> *const core::ffi::c_void {
            let ssp = _sp;
            $(
                let $pname = $crate::read_stack_param!(_sp -> $ptype);
                let _sp = _sp.add(1);
            )*
            let ret = $original( $( $pname ),* );
            $(
                $crate::put_stack_return!(ssp <- ret as $rtype);
            )?
            ffi::m3Err_none as _
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! read_stack_param {
    ($sp:ident -> u64) => {
        *$sp
    };
    ($sp:ident -> u32) => {
        (*$sp & 0xFFFF_FFFF) as u32;
    };
    ($sp:ident -> f64) => {
        f64::from_ne_bytes((*$sp).to_ne_bytes())
    };
    ($sp:ident -> f32) => {
        f64::from_ne_bytes($crate::read_stack_param!($sp -> u32).to_ne_bytes())
    };
    ($sp:ident -> $type:ty) => {
        compile_error!(concat!("unknown function argument type ", stringify!($type)))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! put_stack_return {
    ($sp:ident <- $ident:ident as u64) => {
        *$sp = $ident;
    };
    ($sp:ident -> u32) => {
        *$sp = $ident as u64;
    };
    ($sp:ident -> f64) => {
        *$sp = u64::from_ne_bytes($ident.to_ne_bytes());
    };
    ($sp:ident -> f32) => {
        f64::from_ne_bytes($crate::read_stack_param!($sp -> u32).to_ne_bytes())
        *$sp = u32::from_ne_bytes($ident.to_ne_bytes()) as u64;
    };
    ($sp:ident -> ()) => {};
    ($sp:ident -> $type:ty) => {
        compile_error!(concat!("unknown function argument type ", stringify!($type)))
    };
}
