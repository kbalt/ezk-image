use crate::vector::Vector;

mod from_rgb;
mod read;
mod to_rgb;
mod write;

pub(crate) use from_rgb::RgbToI420;
pub(crate) use read::I420Reader;
pub(crate) use to_rgb::I420ToRgb;
pub(crate) use write::I420Writer;

pub(crate) struct I420Block<V> {
    pub(crate) y00: V,
    pub(crate) y01: V,
    pub(crate) y10: V,
    pub(crate) y11: V,

    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait I420Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V>;
}
