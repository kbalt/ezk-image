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
