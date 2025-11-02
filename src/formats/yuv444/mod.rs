use crate::vector::Vector;

mod from_rgb;
mod to_rgb;

mod read_3plane;
mod write_3plane;

pub(crate) use from_rgb::FromRgb;
pub(crate) use read_3plane::Read3Plane;
pub(crate) use to_rgb::ToRgb;
pub(crate) use write_3plane::Write3Plane;

pub(crate) struct Yuv444Block<V> {
    pub(crate) px00: Yuv444Pixel<V>,
    pub(crate) px01: Yuv444Pixel<V>,
    pub(crate) px10: Yuv444Pixel<V>,
    pub(crate) px11: Yuv444Pixel<V>,
}

pub(crate) struct Yuv444Pixel<V> {
    pub(crate) y: V,
    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait Yuv444Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> Yuv444Block<V>;
}
