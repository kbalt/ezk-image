use crate::bits::BitsInternal;
use crate::formats::{I420Block, I420Src};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct NV12Reader<'a, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    src_y: *const B::Primitive,
    src_uv: *const B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a [B::Primitive]>,
}

impl<'a, B: BitsInternal> NV12Reader<'a, B> {
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

        let PixelFormatPlanes::NV12 { y, uv } = src_planes else {
            panic!("Invalid PixelFormatPlanes for NV12Writer");
        };

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Self {
            window,
            src_width,
            src_y: y.as_ptr(),
            src_uv: uv.as_ptr(),
            max_value: crate::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        }
    }
}

impl<B: BitsInternal> I420Src for NV12Reader<'_, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let uv_offset = ((y / 2) * (self.src_width / 2) + (x / 2)) * 2;

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = B::load::<V>(self.src_y.add(y00_offset));
        let y01 = B::load::<V>(self.src_y.add(y00_offset + V::LEN));
        let y10 = B::load::<V>(self.src_y.add(y10_offset));
        let y11 = B::load::<V>(self.src_y.add(y10_offset + V::LEN));

        // Load U and V
        let uv0 = B::load::<V>(self.src_uv.add(uv_offset));
        let uv1 = B::load::<V>(self.src_uv.add(uv_offset + V::LEN));

        let (u, v) = uv0.unzip(uv1);

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u = u.vdivf(self.max_value);
        let v = v.vdivf(self.max_value);

        I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        }
    }
}
