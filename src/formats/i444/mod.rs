use crate::vector::Vector;

mod from_rgb;
mod read;
mod to_rgb;
mod write;

pub(crate) use from_rgb::RgbToI444;
pub(crate) use read::I444Reader;
pub(crate) use to_rgb::I444ToRgb;
pub(crate) use write::I444Writer;

pub(crate) struct I444Block<V> {
    pub(crate) px00: I444Pixel<V>,
    pub(crate) px01: I444Pixel<V>,
    pub(crate) px10: I444Pixel<V>,
    pub(crate) px11: I444Pixel<V>,
}

pub(crate) struct I444Pixel<V> {
    pub(crate) y: V,
    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait I444Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I444Block<V>;
}
