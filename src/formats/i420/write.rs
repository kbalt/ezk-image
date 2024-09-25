#![allow(clippy::too_many_arguments)]

use super::{I420Block, I420Src};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Window};
use std::marker::PhantomData;

pub(crate) struct I420Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I420Src,
{
    window: Window,
    dst_width: usize,
    dst_y: *mut P,
    dst_u: *mut P,
    dst_v: *mut P,

    max_value: f32,

    i420_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> I420Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I420Src,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [P]>,
        bits_per_component: usize,
        window: Option<Window>,
        i420_src: S,
    ) -> Result<(), ConvertError> {
        if !dst_planes.bounds_check(dst_width, dst_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::I420 { y, u, v } = dst_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::I420));
        };

        visit(
            dst_width,
            dst_height,
            window,
            Self {
                window: window.unwrap_or(Window {
                    x: 0,
                    y: 0,
                    width: dst_width,
                    height: dst_height,
                }),
                dst_width,
                dst_y: y.as_mut_ptr(),
                dst_u: u.as_mut_ptr(),
                dst_v: v.as_mut_ptr(),
                max_value: crate::formats::max_value_for_bits(bits_per_component),
                i420_src,
                _m: PhantomData,
            },
        )
    }
}

impl<P, S> Image2x2Visitor for I420Writer<'_, P, S>
where
    P: PrimitiveInternal,
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
        } = self
            .i420_src
            .read::<V>(x - self.window.x, y - self.window.y);

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);
        let u = u.vmulf(self.max_value);
        let v = v.vmulf(self.max_value);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;
        P::write_2x(self.dst_y.add(offset0), y00, y01);
        P::write_2x(self.dst_y.add(offset1), y10, y11);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        P::write(self.dst_u.add(uv_offset), u);
        P::write(self.dst_v.add(uv_offset), v);
    }
}
