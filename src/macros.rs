//! Public macros

/// A convenience macro for creating a wrapper function that can be linked to wasm3.
///
/// # Example
///
/// ```ignore
/// wasm3::make_func_wrapper!(millis_wrap: millis() -> u64);
/// fn millis() -> u64 {
///     MILLIS
/// }
///
/// module
///     .link_function::<(), u64>("time", "millis", millis_wrap)
///     .expect("Unable to link function");
/// ```
#[macro_export]
macro_rules! make_func_wrapper {
    ( $wis:vis $wrapper_name:ident: $original:ident( $( $pname:ident: $ptype:ident ),* $( , )? ) -> TrappedResult<$rtype:ident>) => {
        $wis unsafe extern "C" fn $wrapper_name(
            _rt: $crate::wasm3_sys::IM3Runtime,
            _sp: $crate::wasm3_sys::m3stack_t,
            _mem: *mut core::ffi::c_void,
        ) -> *const core::ffi::c_void {
            use $crate::WasmType as _;
            let ssp = _sp;
            $(
                let $pname = $ptype::pop_from_stack(_sp);
                let _sp = _sp.add($ptype::SIZE_IN_SLOT_COUNT);
            )*
            let ret = $original( $( $pname ),* );
            match ret {
                Ok(ret) => {
                    $rtype::push_on_stack(ret, ssp);
                    $crate::wasm3_sys::m3Err_none as _
                },
                Err(trap) => trap.as_ptr() as _
            }
        }
    };
    // ptype is an ident because we still want to match on it later -- \/                  rtype too -- \/
    ( $wis:vis $wrapper_name:ident: $original:ident( $( $pname:ident: $ptype:ident ),* $( , )? ) $( -> $rtype:ident )?) => {
        $wis unsafe extern "C" fn $wrapper_name(
            _rt: $crate::wasm3_sys::IM3Runtime,
            _sp: $crate::wasm3_sys::m3stack_t,
            _mem: *mut core::ffi::c_void,
        ) -> *const core::ffi::c_void {
            use $crate::WasmType as _;
            let ssp = _sp;
            $(
                let $pname = $ptype::pop_from_stack(_sp);
                let _sp = _sp.add($ptype::SIZE_IN_SLOT_COUNT);
            )*
            let ret = $original( $( $pname ),* );
            $(
                $rtype::push_on_stack(ret, ssp);
            )?
            $crate::wasm3_sys::m3Err_none as _
        }
    };
}
