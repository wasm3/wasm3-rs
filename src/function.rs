use crate::runtime::Runtime;

pub struct Function<'env, 'rt>(ffi::IM3Function, &'rt Runtime<'env>);

impl<'env, 'rt> Function<'env, 'rt> {
    #[inline]
    pub(crate) fn from_raw(runtime: &'rt Runtime<'env>, ptr: ffi::IM3Function) -> Self {
        Function(ptr, runtime)
    }
}
