// this module looks like a mess, lots of doc(hidden) attributes since rust traits cant have private functions
mod private {
    #[doc(hidden)]
    pub struct Seal;
}

/// Trait implemented by types that can be passed to and from wasm.
pub trait WasmType: Sized {
    #[doc(hidden)]
    const TYPE_INDEX: u8;
    #[doc(hidden)]
    fn put_on_stack(self, stack: &mut [u64]) {
        stack[0] = self.into_u64();
    }
    #[doc(hidden)]
    fn from_u64(val: u64) -> Self;
    #[doc(hidden)]
    fn into_u64(self) -> u64;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

/// Tait implemented by types that can be passed to wasm.
pub trait WasmArg: WasmType {}

/// Helper tait implemented by tuples to emulate "variadic generics".
pub trait WasmArgs {
    #[doc(hidden)]
    fn put_on_stack(self, stack: &mut [u64]);
    #[doc(hidden)]
    // required for closure linking
    fn retrieve_from_stack(stack: &[u64]) -> Self;
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

impl WasmArg for i32 {}
impl WasmType for i32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i32 as u8;
    #[doc(hidden)]
    fn from_u64(val: u64) -> Self {
        (val & 0xFFFF_FFFF) as i32
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
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
    #[doc(hidden)]
    fn from_u64(val: u64) -> Self {
        (val & 0xFFFF_FFFF) as u32
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
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
    fn from_u64(val: u64) -> Self {
        val as i64
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
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
    fn from_u64(val: u64) -> Self {
        val
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
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
    #[doc(hidden)]
    fn from_u64(val: u64) -> Self {
        f32::from_ne_bytes(((val & 0xFFFF_FFFF) as u32).to_ne_bytes())
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        u32::from_ne_bytes(self.to_ne_bytes()) as u64
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
    fn from_u64(val: u64) -> Self {
        f64::from_ne_bytes(val.to_ne_bytes())
    }
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        u64::from_ne_bytes(self.to_ne_bytes())
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmType for () {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_void as u8;
    #[doc(hidden)]
    fn put_on_stack(self, _: &mut [u64]) {}
    #[doc(hidden)]
    fn from_u64(_: u64) -> Self {}
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        unreachable!()
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArgs for () {
    #[doc(hidden)]
    fn put_on_stack(self, _: &mut [u64]) {}
    #[doc(hidden)]
    fn retrieve_from_stack(_: &[u64]) -> Self {}
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
    fn put_on_stack(self, stack: &mut [u64]) {
        WasmType::put_on_stack(self, stack);
    }
    #[doc(hidden)]
    fn retrieve_from_stack(stack: &[u64]) -> Self {
        T::from_u64(stack[0])
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
        impl<$($types,)*> WasmArgs for ($($types,)*)
        where $($types: WasmArg,)* {
            #[doc(hidden)]
            fn put_on_stack(self, stack: &mut [u64]) {
                #[allow(non_snake_case)]
                let ($($types,)*) = self;
                let mut idx = 0;

                $(
                    idx += 1;
                    *stack.get_mut(idx - 1).unwrap() = $types.into_u64();
                )*
            }
            #[doc(hidden)]
            fn retrieve_from_stack(stack: &[u64]) -> Self {
                let mut idx = 0;

                ($(
                    {
                        idx += 1;
                        $types::from_u64(stack[idx - 1])
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
    fn test_put_on_stack_single() {
        let stack = &mut [0u64, 0, 0, 0];
        WasmArgs::put_on_stack(15u64, stack);
        assert_eq!(stack, &[15, 0, 0, 0])
    }
    #[test]
    fn test_put_on_stack_double() {
        let stack = &mut [0u64, 0, 0, 0];
        (15u64, 32u64).put_on_stack(stack);
        assert_eq!(stack, &[15, 32, 0, 0])
    }
    #[test]
    fn test_put_on_stack_quintuple() {
        let stack = &mut [0u64, 0, 0, 0, 0, 0];
        (15u64, 315u64, 0u64, 151_652u64, 32u64).put_on_stack(stack);
        assert_eq!(stack, &[15, 315, 0, 151_652, 32, 0])
    }

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
