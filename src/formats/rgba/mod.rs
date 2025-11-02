use crate::vector::Vector;

mod read;
mod write;

pub(crate) use read::RgbaReader;
pub(crate) use write::RgbaWriter;

pub(crate) const SWIZZLE_RGBA: u8 = 0;
pub(crate) const SWIZZLE_BGRA: u8 = 1;
pub(crate) const SWIZZLE_ARGB: u8 = 2;
pub(crate) const SWIZZLE_ABGR: u8 = 3;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbaPixel<V> {
    pub(crate) r: V,
    pub(crate) g: V,
    pub(crate) b: V,
    pub(crate) a: V,
}

impl<V: Vector> RgbaPixel<V> {
    #[inline(always)]
    fn new(r: V, g: V, b: V, a: V) -> Self {
        Self { r, g, b, a }
    }

    #[inline(always)]
    pub(crate) unsafe fn from_components<const SWIZZLE: u8>(c0: V, c1: V, c2: V, c3: V) -> Self {
        match SWIZZLE {
            SWIZZLE_RGBA => Self::new(c0, c1, c2, c3),
            SWIZZLE_BGRA => Self::new(c2, c1, c0, c3),
            SWIZZLE_ARGB => Self::new(c1, c2, c3, c0),
            SWIZZLE_ABGR => Self::new(c3, c2, c1, c0),
            _ => unreachable!("unknown swizzle {SWIZZLE}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbaBlock<V> {
    pub(crate) px00: RgbaPixel<V>,
    pub(crate) px01: RgbaPixel<V>,
    pub(crate) px10: RgbaPixel<V>,
    pub(crate) px11: RgbaPixel<V>,
}

pub(crate) trait RgbaSrc {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V>;
}
