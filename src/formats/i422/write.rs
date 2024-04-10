#![allow(clippy::too_many_arguments)]

use super::{I422Block, I422Src};
use crate::bits::BitsInternal;
use crate::formats::reader::{read, ImageReader};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I422Writer<'a, B, S>
where
    B: BitsInternal,
    S: I422Src,
{
    dst_width: usize,
    dst_y: *mut B::Primitive,
    dst_u: *mut B::Primitive,
    dst_v: *mut B::Primitive,

    max_value: f32,

    i422_src: S,

    _m: PhantomData<&'a mut [B::Primitive]>,
}

impl<'a, B, S> I422Writer<'a, B, S>
where
    B: BitsInternal,
    S: I422Src,
{
    pub(crate) fn read(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
        i422_src: S,
    ) {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::I422 { y, u, v } = dst_planes else {
            panic!("Invalid PixelFormatPlanes for I422Writer");
        };

        read(
            dst_width,
            dst_height,
            window,
            Self {
                dst_width,
                dst_y: y.as_mut_ptr(),
                dst_u: u.as_mut_ptr(),
                dst_v: v.as_mut_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                i422_src,
                _m: PhantomData,
            },
        )
    }
}

impl<B, S> ImageReader for I422Writer<'_, B, S>
where
    B: BitsInternal,
    S: I422Src,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, x: usize, y: usize) {
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

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;
        B::write_2x(self.dst_y.add(offset0), y00, y01);
        B::write_2x(self.dst_y.add(offset1), y10, y11);

        let hx = x / 2;
        let hw = self.dst_width / 2;

        let uv0_offset = (y * hw) + hx;
        let uv1_offset = ((y + 1) * hw) + hx;

        B::write(self.dst_u.add(uv0_offset), u0);
        B::write(self.dst_u.add(uv1_offset), u1);

        B::write(self.dst_v.add(uv0_offset), v0);
        B::write(self.dst_v.add(uv1_offset), v1);
    }
}
