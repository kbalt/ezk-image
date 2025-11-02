use crate::vector::Vector;

mod from_rgb;
mod read_2plane;
mod read_3plane;
mod to_rgb;
mod write_2plane;
mod write_3plane;

pub(crate) use from_rgb::FromRgb;
pub(crate) use read_2plane::Read2Plane;
pub(crate) use read_3plane::Read3Plane;
pub(crate) use to_rgb::ToRgb;
pub(crate) use write_2plane::Write2Plane;
pub(crate) use write_3plane::Write3Plane;

pub(crate) struct Yuv420Block<V> {
    pub(crate) y00: V,
    pub(crate) y01: V,
    pub(crate) y10: V,
    pub(crate) y11: V,

    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait Yuv420Src {
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> Yuv420Block<V>;
}
