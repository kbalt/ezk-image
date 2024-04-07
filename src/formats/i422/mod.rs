use crate::vector::Vector;

mod from_rgb;
mod read;
mod to_rgb;
mod write;

pub(crate) use from_rgb::RgbToI422;
pub(crate) use read::I422Reader;
pub(crate) use to_rgb::I422ToRgb;
pub(crate) use write::I422Writer;

pub(crate) struct I422Block<V> {
    pub(crate) y00: V,
    pub(crate) y01: V,
    pub(crate) y10: V,
    pub(crate) y11: V,

    pub(crate) u0: V,
    pub(crate) u1: V,
    pub(crate) v0: V,
    pub(crate) v1: V,
}

pub(crate) trait I422Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V>;
}
