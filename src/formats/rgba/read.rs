use super::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Window};
use std::marker::PhantomData;

pub(crate) struct RgbaReader<'a, const ALPHA_FIRST: bool, const BGR: bool, P: PrimitiveInternal> {
    window: Window,

    src_width: usize,
    src: *const P,
    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, const ALPHA_FIRST: bool, const BGR: bool, P: PrimitiveInternal>
    RgbaReader<'a, ALPHA_FIRST, BGR, P>
{
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

        let PixelFormatPlanes::RGBA(src) = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(if ALPHA_FIRST {
                if BGR {
                    PixelFormat::ABGR
                } else {
                    PixelFormat::ARGB
                }
            } else {
                if BGR {
                    PixelFormat::BGRA
                } else {
                    PixelFormat::RGBA
                }
            }));
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
            src: src.as_ptr(),
            max_value: crate::formats::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        })
    }

    #[inline(always)]
    unsafe fn read_rgba_2x<V: Vector>(&mut self, offset: usize) -> (V, V, V, V, V, V, V, V) {
        let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
            P::load_4x_interleaved_2x::<V>(self.src.add(offset * 4));

        if ALPHA_FIRST {
            (a00, r00, g00, b00, a01, r01, g01, b01)
        } else {
            (r00, g00, b00, a00, r01, g01, b01, a01)
        }
    }
}

impl<'a, const ALPHA_FIRST: bool, const BGR: bool, P: PrimitiveInternal> RgbaSrc
    for RgbaReader<'a, ALPHA_FIRST, BGR, P>
{
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let rgba00offset = (y * self.src_width) + x;
        let rgba10offset = ((y + 1) * self.src_width) + x;

        let (r00, g00, b00, a00, r01, g01, b01, a01) = self.read_rgba_2x::<V>(rgba00offset);
        let (r10, g10, b10, a10, r11, g11, b11, a11) = self.read_rgba_2x::<V>(rgba10offset);

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

        let px00 = RgbaPixel::from_loaded::<BGR>(r00, g00, b00, a00);
        let px01 = RgbaPixel::from_loaded::<BGR>(r01, g01, b01, a01);
        let px10 = RgbaPixel::from_loaded::<BGR>(r10, g10, b10, a10);
        let px11 = RgbaPixel::from_loaded::<BGR>(r11, g11, b11, a11);

        RgbaBlock {
            px00,
            px01,
            px10,
            px11,
        }
    }
}
