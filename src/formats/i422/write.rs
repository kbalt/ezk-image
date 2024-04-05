use super::{I422Block, I422Visitor};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I422Writer<'a, B: BitsInternal> {
    window: Rect,

    dst_width: usize,
    y: *mut B::Primitive,
    u: *mut B::Primitive,
    v: *mut B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a mut [B::Primitive]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, B: BitsInternal> I422Writer<'a, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Self {
        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: dst_width,
            height: dst_height,
        });

        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::I422 { y, u, v } = dst_planes else {
            panic!("Invalid PixelFormatPlanes for I422Writer");
        };

        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            y: y.as_mut_ptr(),
            u: u.as_mut_ptr(),
            v: v.as_mut_ptr(),
            max_value: crate::max_value_for_bits(bits_per_component),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<'a, B: BitsInternal> I422Visitor for I422Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: I422Block<V>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let I422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        } = block;

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
        B::write_2x(self.y.add(offset0), y00, y01);
        B::write_2x(self.y.add(offset1), y10, y11);

        let hx = x / 2;
        let hw = self.dst_width / 2;

        let uv0_offset = (y * hw) + hx;
        let uv1_offset = ((y + 1) * hw) + hx;

        B::write(self.u.add(uv0_offset), u0);
        B::write(self.u.add(uv1_offset), u1);

        B::write(self.v.add(uv0_offset), v0);
        B::write(self.v.add(uv1_offset), v1);
    }
}
