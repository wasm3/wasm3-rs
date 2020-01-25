mod private {
    #[doc(hidden)]
    pub struct Seal;
}

pub trait WasmType: Sized /*+ Sealed */ {
    #[doc(hidden)]
    const TYPE_INDEX: u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}
pub trait WasmArg: WasmType {
    #[doc(hidden)]
    fn into_u64(self) -> u64;
}
pub trait WasmArgs {
    #[doc(hidden)]
    fn put_on_stack(self, stack: &mut [u64]);
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool;
    #[doc(hidden)]
    fn sealed_() -> private::Seal;
}

impl WasmType for i32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i32 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        (stack[0] & 0xFFFF_FFFF) as i32
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for i32 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
    }
}

impl WasmType for u32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i32 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        (stack[0] & 0xFFFF_FFFF) as u32
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for u32 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
    }
}

impl WasmType for i64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i64 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        stack[0] as i64
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for i64 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
    }
}

impl WasmType for u64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_i64 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        stack[0]
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for u64 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        self as u64
    }
}

impl WasmType for f32 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_f32 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        f32::from_le_bytes(((stack[0] & 0xFFFF_FFFF) as u32).to_le_bytes())
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for f32 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        u32::from_le_bytes(self.to_le_bytes()) as u64
    }
}

impl WasmType for f64 {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_f64 as u8;
    #[doc(hidden)]
    fn fetch_from_stack(stack: &[u64]) -> Self {
        f64::from_le_bytes(stack[0].to_le_bytes())
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}
impl WasmArg for f64 {
    #[doc(hidden)]
    fn into_u64(self) -> u64 {
        u64::from_le_bytes(self.to_le_bytes())
    }
}

impl WasmType for () {
    #[doc(hidden)]
    const TYPE_INDEX: u8 = ffi::_bindgen_ty_1::c_m3Type_void as u8;
    #[doc(hidden)]
    fn fetch_from_stack(_: &[u64]) -> Self {}
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl WasmArgs for () {
    #[doc(hidden)]
    fn put_on_stack(self, _: &mut [u64]) {}
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool {
        types.is_empty()
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

impl<T> WasmArgs for T
where
    T: WasmArg,
{
    #[doc(hidden)]
    fn put_on_stack(self, stack: &mut [u64]) {
        stack[0] = self.into_u64();
    }
    #[doc(hidden)]
    fn validate_types(types: &[u8]) -> bool {
        types.len() == 1 && types[0] == Self::TYPE_INDEX
    }
    #[doc(hidden)]
    fn sealed_() -> private::Seal {
        private::Seal
    }
}

macro_rules! args_impl {
    ($($types:ident),*) => {
        args_impl!(@rec __DUMMY__T, $($types),*);
    };
    (@rec $types:ident) => {};
    (@rec $_:ident, $($types:ident),*) => {
        impl<$($types,)*> WasmArgs for ($($types,)*)
        where $($types: WasmArg,)*
        {
            #[doc(hidden)]
            fn put_on_stack(self, stack: &mut [u64]) {
                #[allow(non_snake_case)]
                let ($($types,)*) = self;
                let mut stack_iter = stack.iter_mut();
                $(
                    stack_iter.next().map(|place| *place = $types.into_u64());
                )*
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
        args_impl!(@rec $($types),*);
    };
}
args_impl!(A, B, C, D, E, F, G, H, J, K, L, M, N, O, P, Q);
