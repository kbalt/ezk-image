use crate::vector::Vector;

mod read;
mod write;

pub(crate) use read::RgbReader;
pub(crate) use write::RgbWriter;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbPixel<V> {
    pub(crate) r: V,
    pub(crate) g: V,
    pub(crate) b: V,
}

impl<V: Vector> RgbPixel<V> {
    pub(crate) unsafe fn from_loaded<const REVERSE: bool>(r: V, g: V, b: V) -> Self {
        if REVERSE {
            Self { r: b, g, b: r }
        } else {
            Self { r, g, b }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbBlock<V> {
    pub(crate) rgb00: RgbPixel<V>,
    pub(crate) rgb01: RgbPixel<V>,
    pub(crate) rgb10: RgbPixel<V>,
    pub(crate) rgb11: RgbPixel<V>,
}

pub(crate) trait RgbSrc {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V>;
}
