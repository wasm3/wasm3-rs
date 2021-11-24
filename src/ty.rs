use alloc::vec::Vec;

// this module looks like a mess, lots of doc(hidden) attributes since rust traits cant have private functions
mod private {
    #[doc(hidden)]
    pub struct Seal;
}

/// Trait implemented by types that can be passed to and from wasm.
pub trait WasmType: Sized {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize;
    #[doc(hidden)]
    const SIGNATURE: u8;
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self;
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64);
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

/// Tait implemented by types that can be passed to wasm.
pub trait WasmArg: WasmType {}

/// Helper tait implemented by tuples to emulate "variadic generics".
pub trait WasmArgs {
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64);
    #[doc(hidden)]
    // required for closure linking
    unsafe fn pop_from_stack(stack: *mut u64) -> Self;
    #[doc(hidden)]
    fn validate_types(types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
    #[doc(hidden)]
    fn append_signature(buffer: &mut Vec<cty::c_char>);
}

impl WasmArg for i32 {}
impl WasmType for i32 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i32;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'i';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const i32)
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut i32) = self;
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for u32 {}
impl WasmType for u32 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i32;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'i';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const u32)
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut u32) = self;
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for i64 {}
impl WasmType for i64 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i64;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'I';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *(stack as *const i64)
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut i64) = self;
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for u64 {}
impl WasmType for u64 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_i64;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'I';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        *stack
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *stack = self;
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for f32 {}
impl WasmType for f32 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_f32;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'f';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        f32::from_ne_bytes((*(stack as *const u32)).to_ne_bytes())
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *(stack as *mut u32) = u32::from_ne_bytes(self.to_ne_bytes());
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArg for f64 {}
impl WasmType for f64 {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_f64;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 1;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'F';
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        f64::from_ne_bytes((*stack).to_ne_bytes())
    }
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        *stack = u64::from_ne_bytes(self.to_ne_bytes());
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmType for () {
    #[doc(hidden)]
    const TYPE_INDEX: ffi::M3ValueType::Type = ffi::M3ValueType::c_m3Type_none;
    #[doc(hidden)]
    const SIZE_IN_SLOT_COUNT: usize = 0;
    #[doc(hidden)]
    const SIGNATURE: u8 = b'v';
    #[doc(hidden)]
    unsafe fn pop_from_stack(_: *mut u64) -> Self {}
    #[doc(hidden)]
    unsafe fn push_on_stack(self, _: *mut u64) {}
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArgs for () {
    #[doc(hidden)]
    unsafe fn push_on_stack(self, _: *mut u64) {}
    #[doc(hidden)]
    unsafe fn pop_from_stack(_: *mut u64) -> Self {}
    #[doc(hidden)]
    fn validate_types(mut types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool {
        types.next().is_none()
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
    #[doc(hidden)]
    fn append_signature(_buffer: &mut Vec<cty::c_char>) {}
}

/// Unary functions
impl<T> WasmArgs for T
where
    T: WasmArg,
{
    #[doc(hidden)]
    unsafe fn push_on_stack(self, stack: *mut u64) {
        WasmType::push_on_stack(self, stack);
    }
    #[doc(hidden)]
    unsafe fn pop_from_stack(stack: *mut u64) -> Self {
        WasmType::pop_from_stack(stack)
    }
    #[doc(hidden)]
    fn validate_types(mut types: impl Iterator<Item = ffi::M3ValueType::Type>) -> bool {
        types.next().map(|ty| ty == T::TYPE_INDEX).unwrap_or(false) && types.next().is_none()
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
    #[doc(hidden)]
    fn append_signature(buffer: &mut Vec<cty::c_char>) {
        buffer.push(T::SIGNATURE as cty::c_char);
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
            unsafe fn push_on_stack(self, mut stack: *mut u64) {
                #[allow(non_snake_case)]
                let ($($types,)*) = self;

                $(
                    $types.push_on_stack(stack);
                    stack = stack.add($types::SIZE_IN_SLOT_COUNT);
                )*
            }
            #[doc(hidden)]
            unsafe fn pop_from_stack(mut stack: *mut u64) -> Self {
                ($(
                    {
                        let val = $types::pop_from_stack(stack);
                        stack = stack.add($types::SIZE_IN_SLOT_COUNT);
                        val
                    },
                )*)
            }
            #[doc(hidden)]
            fn validate_types(mut types: impl Iterator<Item=ffi::M3ValueType::Type>) -> bool {
                $(
                    types.next().map(|ty| ty == $types::TYPE_INDEX).unwrap_or(false) &&
                )*
                types.next().is_none()
            }
            #[doc(hidden)]
            fn sealed_() -> private::Seal { private::Seal }
            #[doc(hidden)]
            fn append_signature(buffer: &mut Vec<cty::c_char>) {
                $(
                    buffer.push($types::SIGNATURE as cty::c_char);
                )*
            }
        }
    };
}
args_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_types_single() {
        assert!(f64::validate_types(
            [ffi::M3ValueType::c_m3Type_f64,].iter().cloned()
        ));
    }

    #[test]
    fn test_validate_types_single_fail() {
        assert!(!f32::validate_types(
            [ffi::M3ValueType::c_m3Type_f64,].iter().cloned()
        ));
    }

    #[test]
    fn test_validate_types_double() {
        assert!(<(f64, u32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_f64,
                ffi::M3ValueType::c_m3Type_i32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_double_fail() {
        assert!(!<(f32, u64)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_f32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_quintuple() {
        assert!(<(f64, u32, i32, i64, f32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_f64,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_f32,
            ]
            .iter()
            .cloned()
        ));
    }

    #[test]
    fn test_validate_types_quintuple_fail() {
        assert!(!<(f64, u32, i32, i64, f32)>::validate_types(
            [
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_i64,
                ffi::M3ValueType::c_m3Type_i32,
                ffi::M3ValueType::c_m3Type_f32,
                ffi::M3ValueType::c_m3Type_f64,
            ]
            .iter()
            .cloned()
        ));
    }
}
