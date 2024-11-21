use super::{I420Block, I420Src};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct I420Writer<'a, P, S>
where
    P: Primitive,
    S: I420Src,
{
    y: &'a mut [u8],
    u: &'a mut [u8],
    v: &'a mut [u8],

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    i420_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> I420Writer<'a, P, S>
where
    P: Primitive,
    S: I420Src,
{
    pub(crate) fn write(dst: &'a mut dyn ImageMut, i420_src: S) -> Result<(), ConvertError> {
        dst.bounds_check()?;

        let dst_width = dst.width();
        let dst_height = dst.height();
        let dst_format = dst.format();

        let [(y, y_stride), (u, u_stride), (v, v_stride)] = read_planes_mut(dst.planes_mut())?;

        visit(
            dst_width,
            dst_height,
            Self {
                y,
                u,
                v,
                y_stride,
                u_stride,
                v_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                i420_src,
                _m: PhantomData,
            },
        )
    }
}

impl<P, S> Image2x2Visitor for I420Writer<'_, P, S>
where
    P: Primitive,
    S: I420Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = self.i420_src.read::<V>(x, y);

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);

        let u = u.vmulf(self.max_value);
        let v = v.vmulf(self.max_value);

        let y00_offset = y * self.y_stride + x * P::SIZE;
        let y10_offset = (y + 1) * self.y_stride + x * P::SIZE;

        let u_offset = (y / 2) * (self.u_stride) + (x / 2) * P::SIZE;
        let v_offset = (y / 2) * (self.v_stride) + (x / 2) * P::SIZE;

        P::write_2x(&mut self.y[y00_offset..], y00, y01);
        P::write_2x(&mut self.y[y10_offset..], y10, y11);

        P::write(&mut self.u[u_offset..], u);
        P::write(&mut self.v[v_offset..], v);
    }
}
