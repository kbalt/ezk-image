use super::{I420Block, I420Visitor};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I420Writer<'a, B: BitsInternal> {
    window: Rect,

    dst_width: usize,
    y: *mut B::Primitive,
    u: *mut B::Primitive,
    v: *mut B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a mut [B::Primitive]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, B: BitsInternal> I420Writer<'a, B> {
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

        let PixelFormatPlanes::I420 { y, u, v } = dst_planes else {
            panic!("Invalid PixelFormatPlanes for I420Writer");
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

impl<'a, B: BitsInternal> I420Visitor for I420Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: I420Block<V>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);
        let u = u.vmulf(self.max_value);
        let v = v.vmulf(self.max_value);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;
        B::write_2x(self.y.add(offset0), y00, y01);
        B::write_2x(self.y.add(offset1), y10, y11);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        B::write(self.u.add(uv_offset), u);
        B::write(self.v.add(uv_offset), v);
    }
}
