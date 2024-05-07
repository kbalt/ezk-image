use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RgbReader<'a, const REVERSE: bool, P: PrimitiveInternal> {
    window: Rect,

    src_width: usize,
    src: *const P,
    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, const REVERSE: bool, P: PrimitiveInternal> RgbReader<'a, REVERSE, P> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [P]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Result<Self, ConvertError> {
        if !src_planes.bounds_check(src_width, src_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::RGB(src) = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(if REVERSE {
                PixelFormat::BGR
            } else {
                PixelFormat::RGB
            }));
        };

        let window = window.unwrap_or(Rect {
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
            src: src.as_ptr(),
            max_value: crate::formats::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        })
    }
}

impl<'a, const REVERSE: bool, P: PrimitiveInternal> RgbaSrc for RgbReader<'a, REVERSE, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let rgb00offset = (y * self.src_width) + x;
        let rgb10offset = ((y + 1) * self.src_width) + x;

        let [[r00, g00, b00], [r01, g01, b01]] =
            P::load_3x_interleaved_2x::<V>(self.src.add(rgb00offset * 3));
        let [[r10, g10, b10], [r11, g11, b11]] =
            P::load_3x_interleaved_2x::<V>(self.src.add(rgb10offset * 3));

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
