use super::{I422Block, I422Src};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I422Reader<'a, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    y: *const B::Primitive,
    u: *const B::Primitive,
    v: *const B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a [B::Primitive]>,
}

impl<'a, B: BitsInternal> I422Reader<'a, B> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Self {
        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: src_width,
            height: src_height,
        });

        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::I422 { y, u, v } = src_planes else {
            panic!("Invalid PixelFormatPlanes for I422Reader");
        };

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Self {
            window,
            src_width,
            y: y.as_ptr(),
            u: u.as_ptr(),
            v: v.as_ptr(),
            max_value: crate::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        }
    }
}

impl<B: BitsInternal> I422Src for I422Reader<'_, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let uv0_offset = y * (self.src_width / 2) + (x / 2);
        let uv1_offset = (y + 1) * (self.src_width / 2) + (x / 2);

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = B::load::<V>(self.y.add(y00_offset));
        let y01 = B::load::<V>(self.y.add(y00_offset + V::LEN));
        let y10 = B::load::<V>(self.y.add(y10_offset));
        let y11 = B::load::<V>(self.y.add(y10_offset + V::LEN));

        // Load U and V
        let u0 = B::load::<V>(self.u.add(uv0_offset));
        let u1 = B::load::<V>(self.u.add(uv1_offset));

        let v0 = B::load::<V>(self.v.add(uv0_offset));
        let v1 = B::load::<V>(self.v.add(uv1_offset));

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u0 = u0.vdivf(self.max_value);
        let u1 = u1.vdivf(self.max_value);
        let v0 = v0.vdivf(self.max_value);
        let v1 = v1.vdivf(self.max_value);

        I422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        }
    }
}
