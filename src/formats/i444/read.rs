use super::{I444Block, I444Src};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{I444Pixel, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I444Reader<'a, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    y: *const B::Primitive,
    u: *const B::Primitive,
    v: *const B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a [B::Primitive]>,
}

impl<'a, B: BitsInternal> I444Reader<'a, B> {
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

        let PixelFormatPlanes::I444 { y, u, v } = src_planes else {
            panic!("Invalid PixelFormatPlanes for I444Reader");
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

impl<B: BitsInternal> I444Src for I444Reader<'_, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I444Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let px00_offset = (y * self.src_width) + x;
        let px10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = B::load::<V>(self.y.add(px00_offset));
        let y01 = B::load::<V>(self.y.add(px00_offset + V::LEN));
        let y10 = B::load::<V>(self.y.add(px10_offset));
        let y11 = B::load::<V>(self.y.add(px10_offset + V::LEN));

        // Load U pixels
        let u00 = B::load::<V>(self.u.add(px00_offset));
        let u01 = B::load::<V>(self.u.add(px00_offset + V::LEN));
        let u10 = B::load::<V>(self.u.add(px10_offset));
        let u11 = B::load::<V>(self.u.add(px10_offset + V::LEN));

        // Load V pixels
        let v00 = B::load::<V>(self.v.add(px00_offset));
        let v01 = B::load::<V>(self.v.add(px00_offset + V::LEN));
        let v10 = B::load::<V>(self.v.add(px10_offset));
        let v11 = B::load::<V>(self.v.add(px10_offset + V::LEN));

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u00 = u00.vdivf(self.max_value);
        let u01 = u01.vdivf(self.max_value);
        let u10 = u10.vdivf(self.max_value);
        let u11 = u11.vdivf(self.max_value);

        let v00 = v00.vdivf(self.max_value);
        let v01 = v01.vdivf(self.max_value);
        let v10 = v10.vdivf(self.max_value);
        let v11 = v11.vdivf(self.max_value);

        I444Block {
            px00: I444Pixel {
                y: y00,
                u: u00,
                v: v00,
            },
            px01: I444Pixel {
                y: y01,
                u: u01,
                v: v01,
            },
            px10: I444Pixel {
                y: y10,
                u: u10,
                v: v10,
            },
            px11: I444Pixel {
                y: y11,
                u: u11,
                v: v11,
            },
        }
    }
}
