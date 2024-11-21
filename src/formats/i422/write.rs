use super::{I422Block, I422Src};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct I422Writer<'a, P, S>
where
    P: Primitive,
    S: I422Src,
{
    dst_y: *mut u8,
    dst_u: *mut u8,
    dst_v: *mut u8,

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    i422_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> I422Writer<'a, P, S>
where
    P: Primitive,
    S: I422Src,
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
                dst_y: y.as_mut_ptr(),
                dst_u: u.as_mut_ptr(),
                dst_v: v.as_mut_ptr(),
                y_stride,
                u_stride,
                v_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                i422_src,
                _m: PhantomData,
            },
        )
    }
}

impl<P, S> Image2x2Visitor for I422Writer<'_, P, S>
where
    P: Primitive,
    S: I422Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let I422Block {
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

        P::write_2x(self.dst_y.add(y00_offset), y00, y01);
        P::write_2x(self.dst_y.add(y10_offset), y10, y11);

        let u0_offset = (y) * (self.u_stride) + (x / 2) * P::SIZE;
        let u1_offset = (y + 1) * (self.u_stride) + (x / 2) * P::SIZE;

        let v0_offset = (y) * (self.v_stride) + (x / 2) * P::SIZE;
        let v1_offset = (y + 1) * (self.v_stride) + (x / 2) * P::SIZE;

        P::write(self.dst_u.add(u0_offset), u0);
        P::write(self.dst_u.add(u1_offset), u1);

        P::write(self.dst_v.add(v0_offset), v0);
        P::write(self.dst_v.add(v1_offset), v1);
    }
}
