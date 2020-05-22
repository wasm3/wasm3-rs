// this module looks like a mess, lots of doc(hidden) attributes since rust traits cant have private functions
mod private {
    #[doc(hidden)]
    pub struct Seal;
}

#[cfg(feature = "use-32bit-slots")]
#[inline(always)]
unsafe fn read_u32_from_stack(stack: *mut ffi::m3slot_t) -> u32 {
    *stack
}

#[cfg(not(feature = "use-32bit-slots"))]
#[inline(always)]
unsafe fn read_u32_from_stack(stack: *mut ffi::m3slot_t) -> u32 {
    (*stack & 0xFFFF_FFFF) as u32
}

#[cfg(feature = "use-32bit-slots")]
#[inline(always)]
unsafe fn read_u64_from_stack(stack: *mut ffi::m3slot_t) -> u64 {
    *stack.cast::<u64>()
}

#[cfg(not(feature = "use-32bit-slots"))]
#[inline(always)]
unsafe fn read_u64_from_stack(stack: *mut ffi::m3slot_t) -> u64 {
    *stack
}

#[cfg(feature = "use-32bit-slots")]
#[inline(always)]
unsafe fn write_u32_to_stack(stack: *mut ffi::m3slot_t, val: u32) {
    *stack = val;
}

#[cfg(not(feature = "use-32bit-slots"))]
#[inline(always)]
unsafe fn write_u32_to_stack(stack: *mut ffi::m3slot_t, val: u32) {
    *stack.cast::<ffi::m3slot_t>() = val as ffi::m3slot_t;
}

#[cfg(feature = "use-32bit-slots")]
#[inline(always)]
unsafe fn write_u64_to_stack(stack: *mut ffi::m3slot_t, val: u64) {
    *stack.cast::<u64>() = val;
}

#[cfg(not(feature = "use-32bit-slots"))]
#[inline(always)]
unsafe fn write_u64_to_stack(stack: *mut ffi::m3slot_t, val: u64) {
    *stack = val;
}

/// Trait implemented by types that can be passed to and from wasm.
pub trait WasmType: Sized {
    #[doc(hidden)]
    const TYPE_INDEX: u8;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self;
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t);
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

/// Tait implemented by types that can be passed to wasm.
pub trait WasmArg: WasmType {}

/// Helper tait implemented by tuples to emulate "variadic generics".
pub trait WasmArgs {
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut [ffi::m3slot_t]);
    #[doc(hidden)]
    // required for closure linking
    unsafe fn pop_from_stack(stack: *mut [ffi::m3slot_t]) -> Self;
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

impl WasmArg for i32 {}
impl WasmType for i32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i32 as u8;
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2; // cause alignment to u64 boundaries in the stack
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        read_u32_from_stack(stack) as i32
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u32_to_stack(stack, self as u32);
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for u32 {}
impl WasmType for u32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i32 as u8;
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2; // cause alignment to u64 boundaries in the stack
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        read_u32_from_stack(stack)
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u32_to_stack(stack, self);
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for i64 {}
impl WasmType for i64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i64 as u8;
    #[doc(hidden)]
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2;
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;

    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        read_u64_from_stack(stack) as i64
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u64_to_stack(stack, self as u64);
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for u64 {}
impl WasmType for u64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i64 as u8;
    #[doc(hidden)]
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2; // cause alignment to u64 boundaries in the stack
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        read_u64_from_stack(stack)
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u64_to_stack(stack, self);
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for f32 {}
impl WasmType for f32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_f32 as u8;
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2; // cause alignment to u64 boundaries in the stack
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        f32::from_ne_bytes(read_u32_from_stack(stack).to_ne_bytes())
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u32_to_stack(stack, u32::from_ne_bytes(self.to_ne_bytes()));
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for f64 {}
impl WasmType for f64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_f64 as u8;
    #[doc(hidden)]
    #[cfg(feature = "use-32bit-slots")]
    const SIZE_IN_SLOT_COUNT: usize = 2;
    #[doc(hidden)]
    #[cfg(not(feature = "use-32bit-slots"))]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut ffi::m3slot_t) -> Self {
        f64::from_ne_bytes(read_u64_from_stack(stack).to_ne_bytes())
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut ffi::m3slot_t) {
        write_u64_to_stack(stack, u64::from_ne_bytes(self.to_ne_bytes()));
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmType for () {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_none as u8;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 0;
    #[doc(hidden)]
    unsafe fn pop_from_stack(_: *mut ffi::m3slot_t) -> Self {}
    #[doc(hidden)]
    unsafe fn push_on_stack(self, _: *mut ffi::m3slot_t) {}
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArgs for () {
    #[doc(hidden)]
    unsafe fn push_on_stack(self, _: *mut [ffi::m3slot_t]) {}
    #[doc(hidden)]
    unsafe fn pop_from_stack(_: *mut [ffi::m3slot_t]) -> Self {}
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool {
        types.is_empty()
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

/// Unary functions
impl<T> WasmArgs for T
where
    T: WasmArg,
{
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut [ffi::m3slot_t]) {
        WasmType::push_on_stack(self, stack.cast());
    }
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut [ffi::m3slot_t]) -> Self {
        WasmType::pop_from_stack(stack.cast())
    }
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool {
        types
            .get(0)
            .map(|&idx| idx == T::TYPE_INDEX)
            .unwrap_or(false)
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

macro_rules! args_impl {
    ($($types:ident),*) => { args_impl!(@rec [$($types,)*] []); };
    (@rec [] [$($types:ident,)*]) => { args_impl!(@do_impl $($types,)*); };
    (@rec [$head:ident, $($tail:ident,)*] [$($types:ident,)*]) => {
        args_impl!(@do_impl $($types,)*);
        args_impl!(@rec [$($tail,)*] [$($types,)* $head,]);
    };
    (@do_impl) => {/* catch the () case, since its implementation differs slightly */};
    (@do_impl $($types:ident,)*) => {
        #[allow(clippy::eval_order_dependence)]
        #[allow(unused_assignments)]
        impl<$($types,)*> WasmArgs for ($($types,)*)
        where $($types: WasmArg,)* {
            #[doc(hidden)]
            unsafe fn push_on_stack(self, stack: *mut [ffi::m3slot_t]) {
                // reborrowing might be UB here due to aliasing, but there is currently no other stable way to get the metadata of a raw fat pointer
                let mut stack = &mut *stack;
                #[allow(non_snake_case)]
                let ($($types,)*) = self;

                let req_size = 0 $(
                    + $types::SIZE_IN_SLOT_COUNT
                )*;
                assert!(req_size <= stack.len(), "wasm stack was too small");

                $(
                    $types.push_on_stack(stack.as_mut_ptr());
                    stack = &mut stack[$types::SIZE_IN_SLOT_COUNT..];
                )*
            }
            #[doc(hidden)]
            unsafe fn pop_from_stack(stack: *mut [ffi::m3slot_t]) -> Self {
                // reborrowing might be UB here due to aliasing, but there is currently no other stable way to get the metadata of a raw fat pointer
                let mut stack = &mut *stack;
                let req_size = 0 $(
                    + $types::SIZE_IN_SLOT_COUNT
                )*;
                assert!(req_size <= stack.len(), "wasm stack was too small");
                ($(
                    {
                        let val = $types::pop_from_stack(stack.as_mut_ptr());
                        stack = &mut stack[$types::SIZE_IN_SLOT_COUNT..];
                        val
                    },
                )*)
            }
            #[doc(hidden)]
            fn validate_types(types: &[u8]) -> bool {
                let mut ty_iter = types.iter();
                $(
                    ty_iter.next().map(|&ty| ty == $types::TYPE_INDEX).unwrap_or(false)
                )&&*
            }
            #[doc(hidden)]
            fn sealed_() -> private::Seal { private::Seal }
        }
    };
}
args_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_types_single() {
        assert!(f64::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_f64 as u8
        ]));
    }

    #[test]
    fn test_validate_types_single_fail() {
        assert!(!f32::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_f64 as u8
        ]));
    }

    #[test]
    fn test_validate_types_double() {
        assert!(<(f64, u32)>::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_f64 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i32 as u8
        ]));
    }

    #[test]
    fn test_validate_types_double_fail() {
        assert!(!<(f32, u64)>::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_i64 as u8,
            ffi::_bindgen_ty_1::c_m3Type_f32 as u8
        ]));
    }

    #[test]
    fn test_validate_types_quintuple() {
        assert!(<(f64, u32, i32, i64, f32)>::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_f64 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i32 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i32 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i64 as u8,
            ffi::_bindgen_ty_1::c_m3Type_f32 as u8
        ]));
    }

    #[test]
    fn test_validate_types_quintuple_fail() {
        assert!(!<(f64, u32, i32, i64, f32)>::validate_types(&[
            ffi::_bindgen_ty_1::c_m3Type_i32 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i64 as u8,
            ffi::_bindgen_ty_1::c_m3Type_i32 as u8,
            ffi::_bindgen_ty_1::c_m3Type_f32 as u8,
            ffi::_bindgen_ty_1::c_m3Type_f64 as u8
        ]));
    }
}
