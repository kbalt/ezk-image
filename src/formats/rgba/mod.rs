use crate::vector::Vector;

mod read;
mod write;

pub(crate) use read::RgbaReader;
pub(crate) use write::RgbaWriter;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbaPixel<V> {
    pub(crate) r: V,
    pub(crate) g: V,
    pub(crate) b: V,
    pub(crate) a: V,
}

impl<V: Vector> RgbaPixel<V> {
    pub(crate) unsafe fn from_loaded<const REVERSE: bool>(r: V, g: V, b: V, a: V) -> Self {
        if REVERSE {
            Self { r: b, g, b: r, a }
        } else {
            Self { r, g, b, a }
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

#[macro_export]
macro_rules! forward_rgb_rgba {
    () => {
        #[inline(always)]
        unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
            unsafe fn conv<V: Vector>(px: RgbPixel<V>) -> RgbaPixel<V> {
                RgbaPixel {
                    r: px.r,
                    g: px.g,
                    b: px.b,
                    a: V::splat(1.0),
                }
            }

            let block = RgbSrc::read(self, x, y);

            RgbaBlock {
                rgba00: conv(block.rgb00),
                rgba01: conv(block.rgb01),
                rgba10: conv(block.rgb10),
                rgba11: conv(block.rgb11),
            }
        }
    };
}
