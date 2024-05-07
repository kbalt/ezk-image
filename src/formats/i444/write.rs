#![allow(clippy::too_many_arguments)]

use super::{I444Block, I444Src};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I444Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I444Src,
{
    dst_width: usize,
    dst_y: *mut P,
    dst_u: *mut P,
    dst_v: *mut P,

    max_value: f32,

    i444_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> I444Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I444Src,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [P]>,
        bits_per_component: usize,
        window: Option<Rect>,
        i444_src: S,
    ) -> Result<(), ConvertError> {
        if !dst_planes.bounds_check(dst_width, dst_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::I444 { y, u, v } = dst_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::I444));
        };

        visit(
            dst_width,
            dst_height,
            window,
            Self {
                dst_width,
                dst_y: y.as_mut_ptr(),
                dst_u: u.as_mut_ptr(),
                dst_v: v.as_mut_ptr(),
                max_value: crate::formats::max_value_for_bits(bits_per_component),
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

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        P::write_2x(self.dst_y.add(offset0), y00, y01);
        P::write_2x(self.dst_y.add(offset1), y10, y11);

        P::write_2x(self.dst_u.add(offset0), u00, u01);
        P::write_2x(self.dst_u.add(offset1), u10, u11);

        P::write_2x(self.dst_v.add(offset0), v00, v01);
        P::write_2x(self.dst_v.add(offset1), v10, v11);
    }
}
