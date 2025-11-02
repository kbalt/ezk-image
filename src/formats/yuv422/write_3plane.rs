use crate::formats::visit_2x2::{Image2x2Visitor, visit};
use crate::formats::yuv422::{Yuv422Block, Yuv422Src};
use crate::planes::read_planes_mut;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct Write3Plane<'a, P, S>
where
    P: Primitive,
    S: Yuv422Src,
{
    y: &'a mut [u8],
    u: &'a mut [u8],
    v: &'a mut [u8],

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    i422_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> Write3Plane<'a, P, S>
where
    P: Primitive,
    S: Yuv422Src,
{
    pub(crate) fn write(dst: &'a mut dyn ImageMut, i422_src: S) -> Result<(), ConvertError> {
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
                i422_src,
                _m: PhantomData,
            },
        );

        Ok(())
    }
}

impl<P, S> Image2x2Visitor for Write3Plane<'_, P, S>
where
    P: Primitive,
    S: Yuv422Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let Yuv422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        } = self.i422_src.read::<V>(x, y);

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);
        let u0 = u0.vmulf(self.max_value);
        let u1 = u1.vmulf(self.max_value);
        let v0 = v0.vmulf(self.max_value);
        let v1 = v1.vmulf(self.max_value);

        let y00_offset = y * self.y_stride + x * P::SIZE;
        let y10_offset = (y + 1) * self.y_stride + x * P::SIZE;

        P::write_2x(&mut self.y[y00_offset..], y00, y01);
        P::write_2x(&mut self.y[y10_offset..], y10, y11);

        let u0_offset = (y) * (self.u_stride) + (x / 2) * P::SIZE;
        let u1_offset = (y + 1) * (self.u_stride) + (x / 2) * P::SIZE;

        let v0_offset = (y) * (self.v_stride) + (x / 2) * P::SIZE;
        let v1_offset = (y + 1) * (self.v_stride) + (x / 2) * P::SIZE;

        P::write(&mut self.u[u0_offset..], u0);
        P::write(&mut self.u[u1_offset..], u1);

        P::write(&mut self.v[v0_offset..], v0);
        P::write(&mut self.v[v1_offset..], v1);
    }
}
