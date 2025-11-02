use crate::vector::Vector;

mod read_rgb;
mod read_rgba;
mod write_rgb;
mod write_rgba;

pub(crate) type ReadRgba<'a, P> = read_rgba::ReadRgba<'a, SWIZZLE_RGBA, P>;
pub(crate) type ReadBgra<'a, P> = read_rgba::ReadRgba<'a, SWIZZLE_BGRA, P>;
pub(crate) type ReadArgb<'a, P> = read_rgba::ReadRgba<'a, SWIZZLE_ARGB, P>;
pub(crate) type ReadAbgr<'a, P> = read_rgba::ReadRgba<'a, SWIZZLE_ABGR, P>;

pub(crate) type WriteRgba<'a, P, S> = write_rgba::WriteRgba<'a, SWIZZLE_RGBA, P, S>;
pub(crate) type WriteBgra<'a, P, S> = write_rgba::WriteRgba<'a, SWIZZLE_BGRA, P, S>;
pub(crate) type WriteArgb<'a, P, S> = write_rgba::WriteRgba<'a, SWIZZLE_ARGB, P, S>;
pub(crate) type WriteAbgr<'a, P, S> = write_rgba::WriteRgba<'a, SWIZZLE_ABGR, P, S>;

pub(crate) type ReadRgb<'a, P> = read_rgb::ReadRgb<'a, SWIZZLE_RGBA, P>;
pub(crate) type ReadBgr<'a, P> = read_rgb::ReadRgb<'a, SWIZZLE_BGRA, P>;

pub(crate) type WriteRgb<'a, P, S> = write_rgb::WriteRgb<'a, SWIZZLE_RGBA, P, S>;
pub(crate) type WriteBgr<'a, P, S> = write_rgb::WriteRgb<'a, SWIZZLE_BGRA, P, S>;

const SWIZZLE_RGBA: u8 = 0;
const SWIZZLE_BGRA: u8 = 1;
const SWIZZLE_ARGB: u8 = 2;
const SWIZZLE_ABGR: u8 = 3;

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
