use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I422Block, I422Src, PixelFormat, PixelFormatPlanes, Window};
use std::marker::PhantomData;

use super::PackedYuv422Format;

pub(crate) struct PackedYuv422Reader<'a, const FORMAT: u8, P: PrimitiveInternal> {
    window: Window,

    src_width: usize,
    packed: *const P,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, const FORMAT: u8, P: PrimitiveInternal> PackedYuv422Reader<'a, FORMAT, P> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [P]>,
        bits_per_component: usize,
        window: Option<Window>,
    ) -> Result<Self, ConvertError> {
        if !src_planes.bounds_check(src_width, src_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::PackedYuv422(packed) = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::YUYV));
        };

        let window = window.unwrap_or(Window {
            x: 0,
            y: 0,
            width: src_width,
            height: src_height,
        });

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Ok(Self {
            window,
            src_width,
            packed: packed.as_ptr(),
            max_value: crate::formats::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        })
    }

    #[inline(always)]
    unsafe fn read_yuv<V: Vector>(&mut self, offset: usize) -> (V, V) {
        let p00 = P::load::<V>(self.packed.add(offset));
        let p01 = P::load::<V>(self.packed.add(offset + V::LEN));

        let (a, b) = p00.unzip(p01);

        if FORMAT == PackedYuv422Format::YUYV as u8 {
            (a, b)
        } else if FORMAT == PackedYuv422Format::UYVY as u8 {
            (b, a)
        } else {
            unreachable!()
        }
    }
}

impl<const FORMAT: u8, P: PrimitiveInternal> I422Src for PackedYuv422Reader<'_, FORMAT, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset0 = (y * self.src_width) + x;
        let offset1 = ((y + 1) * self.src_width) + x;

        let (y00, uv00) = self.read_yuv::<V>(offset0 * 2);
        let (y01, uv01) = self.read_yuv::<V>((offset0 + V::LEN) * 2);
        let (y10, uv10) = self.read_yuv::<V>(offset1 * 2);
        let (y11, uv11) = self.read_yuv::<V>((offset1 + V::LEN) * 2);

        let (u0, v0) = uv00.unzip(uv01);
        let (u1, v1) = uv10.unzip(uv11);

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
