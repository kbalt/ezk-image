use super::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef};
use std::marker::PhantomData;

pub(crate) struct RgbaReader<'a, const REVERSE: bool, P: PrimitiveInternal> {
    rgba: *const u8,

    rgba_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, const REVERSE: bool, P: PrimitiveInternal> RgbaReader<'a, REVERSE, P> {
    pub(crate) fn new(src: impl ImageRef<'a>) -> Result<Self, ConvertError> {
        if !src.bounds_check() {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let [rgb] = src.planes() else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(src.format()));
        };

        let [rgba_stride] = *src.strides() else {
            return Err(ConvertError::InvalidStridesForPixelFormat(src.format()));
        };

        Ok(Self {
            rgba: rgb.as_ptr(),
            rgba_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }
}

impl<'a, const REVERSE: bool, B: PrimitiveInternal> RgbaSrc for RgbaReader<'a, REVERSE, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let rgba00offset = ((y * self.rgba_stride) + x) * 4;
        let rgba10offset = (((y + 1) * self.rgba_stride) + x) * 4;

        let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
            B::load_4x_interleaved_2x::<V>(self.rgba.add(rgba00offset));
        let [[r10, g10, b10, a10], [r11, g11, b11, a11]] =
            B::load_4x_interleaved_2x::<V>(self.rgba.add(rgba10offset));

        let r00 = r00.vdivf(self.max_value);
        let g00 = g00.vdivf(self.max_value);
        let b00 = b00.vdivf(self.max_value);
        let a00 = a00.vdivf(self.max_value);

        let r01 = r01.vdivf(self.max_value);
        let g01 = g01.vdivf(self.max_value);
        let b01 = b01.vdivf(self.max_value);
        let a01 = a01.vdivf(self.max_value);

        let r10 = r10.vdivf(self.max_value);
        let g10 = g10.vdivf(self.max_value);
        let b10 = b10.vdivf(self.max_value);
        let a10 = a10.vdivf(self.max_value);

        let r11 = r11.vdivf(self.max_value);
        let g11 = g11.vdivf(self.max_value);
        let b11 = b11.vdivf(self.max_value);
        let a11 = a11.vdivf(self.max_value);

        let px00 = RgbaPixel::from_loaded::<REVERSE>(r00, g00, b00, a00);
        let px01 = RgbaPixel::from_loaded::<REVERSE>(r01, g01, b01, a01);
        let px10 = RgbaPixel::from_loaded::<REVERSE>(r10, g10, b10, a10);
        let px11 = RgbaPixel::from_loaded::<REVERSE>(r11, g11, b11, a11);

        RgbaBlock {
            px00,
            px01,
            px10,
            px11,
        }
    }
}
