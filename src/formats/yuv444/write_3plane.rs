use crate::formats::visit_2x2::{Image2x2Visitor, visit};
use crate::formats::yuv444::{Yuv444Block, Yuv444Src};
use crate::planes::read_planes_mut;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct Write3Plane<'a, P, S>
where
    P: Primitive,
    S: Yuv444Src,
{
    y: &'a mut [u8],
    u: &'a mut [u8],
    v: &'a mut [u8],

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    yuv444_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> Write3Plane<'a, P, S>
where
    P: Primitive,
    S: Yuv444Src,
{
    pub(crate) fn write(dst: &'a mut dyn ImageMut, i444_src: S) -> Result<(), ConvertError> {
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
                yuv444_src: i444_src,
                _m: PhantomData,
            },
        );

        Ok(())
    }
}

impl<P, S> Image2x2Visitor for Write3Plane<'_, P, S>
where
    P: Primitive,
    S: Yuv444Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.yuv444_src.read::<V>(x, y);

        let Yuv444Block {
            px00,
            px01,
            px10,
            px11,
        } = block;

        let y00 = px00.y.vmulf(self.max_value);
        let y01 = px01.y.vmulf(self.max_value);
        let y10 = px10.y.vmulf(self.max_value);
        let y11 = px11.y.vmulf(self.max_value);

        let u00 = px00.u.vmulf(self.max_value);
        let u01 = px01.u.vmulf(self.max_value);
        let u10 = px10.u.vmulf(self.max_value);
        let u11 = px11.u.vmulf(self.max_value);

        let v00 = px00.v.vmulf(self.max_value);
        let v01 = px01.v.vmulf(self.max_value);
        let v10 = px10.v.vmulf(self.max_value);
        let v11 = px11.v.vmulf(self.max_value);

        let y00_offset = y * self.y_stride + x * P::SIZE;
        let y10_offset = (y + 1) * self.y_stride + x * P::SIZE;

        let u00_offset = y * self.u_stride + x * P::SIZE;
        let u10_offset = (y + 1) * self.u_stride + x * P::SIZE;

        let v00_offset = y * self.v_stride + x * P::SIZE;
        let v10_offset = (y + 1) * self.v_stride + x * P::SIZE;

        P::write_2x(&mut self.y[y00_offset..], y00, y01);
        P::write_2x(&mut self.y[y10_offset..], y10, y11);

        P::write_2x(&mut self.u[u00_offset..], u00, u01);
        P::write_2x(&mut self.u[u10_offset..], u10, u11);

        P::write_2x(&mut self.v[v00_offset..], v00, v01);
        P::write_2x(&mut self.v[v10_offset..], v10, v11);
    }
}
