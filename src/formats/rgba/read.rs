use super::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::bits::BitsInternal;
use crate::formats::rgb::{RgbBlock, RgbPixel, RgbSrc};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RgbaReader<'a, const REVERSE: bool, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    src: *const B::Primitive,
    max_value: f32,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, const REVERSE: bool, B: BitsInternal> RgbaReader<'a, REVERSE, B> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Self {
        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::RGBA(src) = src_planes else {
            panic!("Invalid PixelFormatPlanes for RgbaReader");
        };

        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: src_width,
            height: src_height,
        });

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Self {
            window,
            src_width,
            src: src.as_ptr(),
            max_value: crate::max_value_for_bits(bits_per_component),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<const REVERSE: bool, B> RgbSrc for RgbaReader<'_, REVERSE, B>
where
    Self: RgbaSrc,
    B: BitsInternal,
{
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        unsafe fn conv<V: Vector>(px: RgbaPixel<V>) -> RgbPixel<V> {
            RgbPixel {
                r: px.r,
                g: px.g,
                b: px.b,
            }
        }

        let block = RgbaSrc::read(self, x, y);

        RgbBlock {
            rgb00: conv(block.rgba00),
            rgb01: conv(block.rgba01),
            rgb10: conv(block.rgba10),
            rgb11: conv(block.rgba11),
        }
    }
}

impl<const REVERSE: bool, B: BitsInternal> RgbaSrc for RgbaReader<'_, REVERSE, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let rgba00offset = (y * self.src_width) + x;
        let rgba10offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<REVERSE, B, V>(self.src, rgba00offset, rgba10offset, self.max_value)
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn load_and_visit_block<const REVERSE: bool, B, V>(
    src_ptr: *const B::Primitive,
    rgba00offset: usize,
    rgba10offset: usize,
    max_value: f32,
) -> RgbaBlock<V>
where
    B: BitsInternal,
    V: Vector,
{
    let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
        B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba00offset * 4));
    let [[r10, g10, b10, a10], [r11, g11, b11, a11]] =
        B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba10offset * 4));

    let r00 = r00.vdivf(max_value);
    let g00 = g00.vdivf(max_value);
    let b00 = b00.vdivf(max_value);
    let a00 = a00.vdivf(max_value);

    let r01 = r01.vdivf(max_value);
    let g01 = g01.vdivf(max_value);
    let b01 = b01.vdivf(max_value);
    let a01 = a01.vdivf(max_value);

    let r10 = r10.vdivf(max_value);
    let g10 = g10.vdivf(max_value);
    let b10 = b10.vdivf(max_value);
    let a10 = a10.vdivf(max_value);

    let r11 = r11.vdivf(max_value);
    let g11 = g11.vdivf(max_value);
    let b11 = b11.vdivf(max_value);
    let a11 = a11.vdivf(max_value);

    let px00 = RgbaPixel::from_loaded::<REVERSE>(r00, g00, b00, a00);
    let px01 = RgbaPixel::from_loaded::<REVERSE>(r01, g01, b01, a01);
    let px10 = RgbaPixel::from_loaded::<REVERSE>(r10, g10, b10, a10);
    let px11 = RgbaPixel::from_loaded::<REVERSE>(r11, g11, b11, a11);

    RgbaBlock {
        rgba00: px00,
        rgba01: px01,
        rgba10: px10,
        rgba11: px11,
    }
}
