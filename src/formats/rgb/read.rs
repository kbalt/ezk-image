use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::planes::read_planes;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct RgbReader<'a, const REVERSE: bool, P: Primitive> {
    rgb: &'a [u8],

    rgb_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, const REVERSE: bool, P: Primitive> RgbReader<'a, REVERSE, P> {
    pub(crate) fn new(src: &'a dyn ImageRef) -> Result<Self, ConvertError> {
        src.bounds_check()?;

        let [(rgb, rgb_stride)] = read_planes(src.planes())?;

        Ok(Self {
            rgb,
            rgb_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }
}

impl<const REVERSE: bool, P: Primitive> RgbaSrc for RgbReader<'_, REVERSE, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let rgb00offset = y * self.rgb_stride + x * 3 * P::SIZE;
        let rgb10offset = (y + 1) * self.rgb_stride + x * 3 * P::SIZE;

        let [[r00, g00, b00], [r01, g01, b01]] =
            P::load_3x_interleaved_2x::<V>(&self.rgb[rgb00offset..]);
        let [[r10, g10, b10], [r11, g11, b11]] =
            P::load_3x_interleaved_2x::<V>(&self.rgb[rgb10offset..]);

        let r00 = r00.vdivf(self.max_value);
        let g00 = g00.vdivf(self.max_value);
        let b00 = b00.vdivf(self.max_value);

        let r01 = r01.vdivf(self.max_value);
        let g01 = g01.vdivf(self.max_value);
        let b01 = b01.vdivf(self.max_value);

        let r10 = r10.vdivf(self.max_value);
        let g10 = g10.vdivf(self.max_value);
        let b10 = b10.vdivf(self.max_value);

        let r11 = r11.vdivf(self.max_value);
        let g11 = g11.vdivf(self.max_value);
        let b11 = b11.vdivf(self.max_value);

        let px00 = RgbaPixel::from_loaded::<REVERSE>(r00, g00, b00, <V>::splat(1.0));
        let px01 = RgbaPixel::from_loaded::<REVERSE>(r01, g01, b01, <V>::splat(1.0));
        let px10 = RgbaPixel::from_loaded::<REVERSE>(r10, g10, b10, <V>::splat(1.0));
        let px11 = RgbaPixel::from_loaded::<REVERSE>(r11, g11, b11, <V>::splat(1.0));

        RgbaBlock {
            px00,
            px01,
            px10,
            px11,
        }
    }
}
