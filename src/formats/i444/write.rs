use super::{I444Block, I444Visitor};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I444Writer<'a, B: BitsInternal> {
    window: Rect,

    dst_width: usize,
    y: *mut B::Primitive,
    u: *mut B::Primitive,
    v: *mut B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a mut [B::Primitive]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, B: BitsInternal> I444Writer<'a, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
        bits_per_channel: usize,
        window: Option<Rect>,
    ) -> Self {
        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: dst_width,
            height: dst_height,
        });

        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::I444 { y, u, v } = dst_planes else {
            panic!("Invalid PixelFormatPlanes for I444Writer");
        };

        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            y: y.as_mut_ptr(),
            u: u.as_mut_ptr(),
            v: v.as_mut_ptr(),
            max_value: crate::max_value_for_bits(bits_per_channel),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<'a, B: BitsInternal> I444Visitor for I444Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: I444Block<V>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

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

        B::write_2x(self.y.add(offset0), y00, y01);
        B::write_2x(self.y.add(offset1), y10, y11);

        B::write_2x(self.u.add(offset0), u00, u01);
        B::write_2x(self.u.add(offset1), u10, u11);

        B::write_2x(self.v.add(offset0), v00, v01);
        B::write_2x(self.v.add(offset1), v10, v11);
    }
}
