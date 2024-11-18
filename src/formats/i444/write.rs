use super::{I444Block, I444Src};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct I444Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I444Src,
{
    dst_y: *mut u8,
    dst_u: *mut u8,
    dst_v: *mut u8,

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    i444_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> I444Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I444Src,
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
                dst_y: y.as_mut_ptr(),
                dst_u: u.as_mut_ptr(),
                dst_v: v.as_mut_ptr(),
                y_stride,
                u_stride,
                v_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                i444_src,
                _m: PhantomData,
            },
        )
    }
}

impl<P, S> Image2x2Visitor for I444Writer<'_, P, S>
where
    P: PrimitiveInternal,
    S: I444Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.i444_src.read::<V>(x, y);

        let I444Block {
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

        P::write_2x(self.dst_y.add(y00_offset), y00, y01);
        P::write_2x(self.dst_y.add(y10_offset), y10, y11);

        P::write_2x(self.dst_u.add(u00_offset), u00, u01);
        P::write_2x(self.dst_u.add(u10_offset), u10, u11);

        P::write_2x(self.dst_v.add(v00_offset), v00, v01);
        P::write_2x(self.dst_v.add(v10_offset), v10, v11);
    }
}
