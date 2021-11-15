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
    ( $wis:vis $wrapper_name:ident: $original:ident( $( $pname:ident: $ptype:ty ),* $( , )? ) -> TrappedResult<$rtype:ty>) => {
        $wis unsafe extern "C" fn $wrapper_name(
            _rt: $crate::wasm3_sys::IM3Runtime,
            _ctx: $crate::wasm3_sys::IM3ImportContext,
            sp: *mut u64,
            _mem: *mut core::ffi::c_void,
        ) -> *const core::ffi::c_void {
            use $crate::WasmType as _;
            let mut _argp = sp.add(<$rtype>::SIZE_IN_SLOT_COUNT);
            $(
                let $pname = <$ptype as $crate::WasmType>::pop_from_stack(_argp);
                _argp = _argp.add(<$ptype>::SIZE_IN_SLOT_COUNT);
            )*
            let ret = $original( $( $pname ),* );
            match ret {
                Ok(ret) => {
                    <$rtype as $crate::WasmType>::push_on_stack(ret, sp);
                    $crate::wasm3_sys::m3Err_none as _
                },
                Err(trap) => trap.as_ptr() as _
            }
        }
    };
    // ptype is an ident because we still want to match on it later -- \/                  rtype too -- \/
    ( $wis:vis $wrapper_name:ident: $original:ident( $( $pname:ident: $ptype:ty ),* $( , )? ) $( -> $rtype:ty )?) => {
        $wis unsafe extern "C" fn $wrapper_name(
            _rt: $crate::wasm3_sys::IM3Runtime,
            _ctx: $crate::wasm3_sys::IM3ImportContext,
            sp: *mut u64,
            _mem: *mut core::ffi::c_void,
        ) -> *const core::ffi::c_void {
            use $crate::WasmType as _;
            let mut _argp = sp;
            $(
                _argp = _argp.add(<$rtype>::SIZE_IN_SLOT_COUNT);
            )?
            $(
                let $pname = <$ptype as $crate::WasmType>::pop_from_stack(_argp);
                _argp = _argp.add(<$ptype>::SIZE_IN_SLOT_COUNT);
            )*
            let _ret = $original( $( $pname ),* );
            $(
                <$rtype as $crate::WasmType>::push_on_stack(_ret, sp);
            )?
            $crate::wasm3_sys::m3Err_none as _
        }
    };
}
