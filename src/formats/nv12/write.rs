#![allow(clippy::too_many_arguments)]

use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::formats::{I420Block, I420Src};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct NV12Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I420Src,
{
    dst_width: usize,
    dst_y: *mut P,
    dst_uv: *mut P,

    max_value: f32,

    i420_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> NV12Writer<'a, P, S>
where
    P: PrimitiveInternal,
    S: I420Src,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [P]>,
        bits_per_component: usize,
        window: Option<Rect>,
        i420_src: S,
    ) {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::NV12 { y, uv } = dst_planes else {
            panic!("Invalid PixelFormatPlanes for read_nv12");
        };

        visit(
            dst_width,
            dst_height,
            window,
            Self {
                dst_width,
                dst_y: y.as_mut_ptr(),
                dst_uv: uv.as_mut_ptr(),
                max_value: crate::formats::max_value_for_bits(bits_per_component),
                i420_src,
                _m: PhantomData,
            },
        )
    }
}

impl<P, S> Image2x2Visitor for NV12Writer<'_, P, S>
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
        } = self.i420_src.read::<V>(x, y);

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

        let (uv0, uv1) = u.zip(v);

        let uv_offset = (hy * hw) + hx;
        P::write_2x(self.dst_uv.add(uv_offset * 2), uv0, uv1);
    }
}
