use crate::vector::Vector;
use crate::{RgbaBlock, RgbaSrc};

pub(crate) trait DynRgbaReaderSpec<V> {
    unsafe fn dyn_read(&mut self, x: usize, y: usize) -> RgbaBlock<V>;
}

pub(crate) use platform::DynRgbaReader;

// x86, x86_64
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod platform {
    use super::DynRgbaReaderSpec;
    use crate::{arch::*, RgbaBlock, RgbaSrc};

    pub(crate) trait DynRgbaReader:
        DynRgbaReaderSpec<f32> + DynRgbaReaderSpec<__m256>
    {
    }

    impl<R: DynRgbaReaderSpec<f32> + DynRgbaReaderSpec<__m256>> DynRgbaReader for R {}

    impl<R: RgbaSrc> DynRgbaReaderSpec<__m256> for R {
        #[target_feature(enable = "avx2", enable = "fma")]
        unsafe fn dyn_read(&mut self, x: usize, y: usize) -> RgbaBlock<__m256> {
            <R as RgbaSrc>::read(self, x, y)
        }
    }
}

// aarch64
#[cfg(target_arch = "aarch64")]
mod platform {
    use super::DynRgbaReaderSpec;
    use crate::{arch::*, RgbaBlock, RgbaSrc};

    pub(crate) trait DynRgbaReader:
        DynRgbaReaderSpec<f32> + DynRgbaReaderSpec<float32x4_t>
    {
    }

    impl<R: DynRgbaReaderSpec<f32> + DynRgbaReaderSpec<float32x4_t>> DynRgbaReader for R {}

    impl<R: RgbaSrc> DynRgbaReaderSpec<float32x4_t> for R {
        #[target_feature(enable = "neon")]
        unsafe fn dyn_read(&mut self, x: usize, y: usize) -> RgbaBlock<float32x4_t> {
            <R as RgbaSrc>::read(self, x, y)
        }
    }
}

// fallback
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
mod platform {
    use super::DynRgbaReaderSpec;
    use crate::vector::Vector;
    use crate::{RgbaBlock, RgbaSrc};

    pub(crate) trait DynRgbaReader: DynRgbaReaderSpec<f32> {}

    impl<R: DynRgbaReaderSpec<f32>> DynRgbaReader for R {}
}

impl<R: RgbaSrc> DynRgbaReaderSpec<f32> for R {
    unsafe fn dyn_read(&mut self, x: usize, y: usize) -> RgbaBlock<f32> {
        <R as RgbaSrc>::read(self, x, y)
    }
}

impl<'a> RgbaSrc for Box<dyn DynRgbaReader + 'a> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        V::dyn_rgba_read(&mut **self, x, y)
    }
}
