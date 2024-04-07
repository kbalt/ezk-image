use super::{RgbBlock, RgbPixel, RgbSrc};
use crate::bits::BitsInternal;
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RgbReader<'a, const REVERSE: bool, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    src: *const B::Primitive,
    max_value: f32,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, const REVERSE: bool, B: BitsInternal> RgbReader<'a, REVERSE, B> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Self {
        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::RGB(src) = src_planes else {
            panic!("Invalid PixelFormatPlanes for RgbReader");
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

impl<const REVERSE: bool, B> RgbaSrc for RgbReader<'_, REVERSE, B>
where
    B: BitsInternal,
{
    forward_rgb_rgba!();
}

impl<const REVERSE: bool, B: BitsInternal> RgbSrc for RgbReader<'_, REVERSE, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let rgb00offset = (y * self.src_width) + x;
        let rgb10offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<REVERSE, B, V>(self.src, rgb00offset, rgb10offset, self.max_value)
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn load_and_visit_block<const REVERSE: bool, B, V>(
    src_ptr: *const B::Primitive,
    rgb00offset: usize,
    rgb10offset: usize,
    max_value: f32,
) -> RgbBlock<V>
where
    B: BitsInternal,
    V: Vector,
{
    let [[r00, g00, b00], [r01, g01, b01]] =
        B::load_3x_interleaved_2x::<V>(src_ptr.add(rgb00offset * 3));
    let [[r10, g10, b10], [r11, g11, b11]] =
        B::load_3x_interleaved_2x::<V>(src_ptr.add(rgb10offset * 3));

    let r00 = r00.vdivf(max_value);
    let g00 = g00.vdivf(max_value);
    let b00 = b00.vdivf(max_value);

    let r01 = r01.vdivf(max_value);
    let g01 = g01.vdivf(max_value);
    let b01 = b01.vdivf(max_value);

    let r10 = r10.vdivf(max_value);
    let g10 = g10.vdivf(max_value);
    let b10 = b10.vdivf(max_value);

    let r11 = r11.vdivf(max_value);
    let g11 = g11.vdivf(max_value);
    let b11 = b11.vdivf(max_value);

    let px00 = RgbPixel::from_loaded::<REVERSE>(r00, g00, b00);
    let px01 = RgbPixel::from_loaded::<REVERSE>(r01, g01, b01);
    let px10 = RgbPixel::from_loaded::<REVERSE>(r10, g10, b10);
    let px11 = RgbPixel::from_loaded::<REVERSE>(r11, g11, b11);

    RgbBlock {
        rgb00: px00,
        rgb01: px01,
        rgb10: px10,
        rgb11: px11,
    }
}
