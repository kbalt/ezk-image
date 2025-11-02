use crate::vector::Vector;

mod from_rgb;
mod read_1plane;
mod read_3plane;
mod to_rgb;
mod write_1plane;
mod write_3plane;

pub(crate) use from_rgb::FromRgb;
pub(crate) use read_1plane::Read1Plane;
pub(crate) use read_3plane::Read3Plane;
pub(crate) use to_rgb::ToRgb;
pub(crate) use write_1plane::Write1Plane;
pub(crate) use write_3plane::Write3Plane;

pub(crate) struct Yuv422Block<V> {
    pub(crate) y00: V,
    pub(crate) y01: V,
    pub(crate) y10: V,
    pub(crate) y11: V,

    pub(crate) u0: V,
    pub(crate) u1: V,
    pub(crate) v0: V,
    pub(crate) v1: V,
}

pub(crate) trait Yuv422Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> Yuv422Block<V>;
}
