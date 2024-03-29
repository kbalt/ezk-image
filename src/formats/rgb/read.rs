use super::{RgbBlock, RgbBlockVisitor, RgbBlockVisitorImpl, RgbPixel};
use crate::arch::*;
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};

#[inline(never)]
pub(crate) fn read_rgb_4x<const REVERSE: bool, B, Vis>(
    src_width: usize,
    src_height: usize,
    src_planes: PixelFormatPlanes<&[B::Primitive]>,
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: RgbBlockVisitor,
{
    assert!(src_planes.bounds_check(src_width, src_height));

    let PixelFormatPlanes::RGB(src) = src_planes else {
        panic!("Invalid PixelFormatPlanes for read_rgb");
    };

    let window = window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src_width,
        height: src_height,
    });

    assert!((window.x + window.width) <= src_width);
    assert!((window.y + window.height) <= src_height);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        #[target_feature(enable = "avx2")]
        unsafe fn call<const REVERSE: bool, B, Vis>(
            src_width: usize,
            src: &[B::Primitive],
            bits_per_channel: usize,
            window: Rect,
            visitor: Vis,
        ) where
            B: BitsInternal,
            Vis: RgbBlockVisitor,
        {
            read_rgb_4x_impl::<REVERSE, B, __m256, _>(
                src_width,
                src,
                bits_per_channel,
                window,
                visitor,
            );
        }

        // Safety: Did a feature check
        unsafe {
            return call::<REVERSE, B, _>(src_width, src, bits_per_channel, window, visitor);
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        #[target_feature(enable = "neon")]
        unsafe fn call<const REVERSE: bool, B, Vis>(
            src_width: usize,
            src: &[B::Primitive],
            bits_per_channel: usize,
            window: Rect,
            visitor: Vis,
        ) where
            B: BitsInternal,
            Vis: RgbBlockVisitor,
        {
            read_rgb_4x_impl::<REVERSE, B, float32x4_t, _>(
                src_width,
                src,
                bits_per_channel,
                window,
                visitor,
            );
        }

        // Safety: Did a feature check
        unsafe {
            return call::<REVERSE, B, _>(src_width, src, bits_per_channel, window, visitor);
        }
    }

    // Fallback to naive
    // Safety: Inputs have been checked
    unsafe {
        read_rgb_4x_impl::<REVERSE, B, f32, _>(src_width, src, bits_per_channel, window, visitor)
    }
}

#[inline(always)]
unsafe fn read_rgb_4x_impl<const REVERSE: bool, B, V, Vis>(
    src_width: usize,
    src: &[B::Primitive],
    bits_per_channel: usize,
    window: Rect,
    mut visitor: Vis,
) where
    B: BitsInternal,
    V: Vector,
    Vis: RgbBlockVisitor + RgbBlockVisitorImpl<V>,
{
    let max_value = crate::max_value_for_bits(bits_per_channel);

    let non_vectored_pixels_per_row = window.width % (V::LEN * 2);
    let vectorized_pixels_per_row = window.width - non_vectored_pixels_per_row;

    let src_ptr = src.as_ptr();

    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        for x in (0..vectorized_pixels_per_row).step_by(V::LEN * 2) {
            let x = window.x + x;

            let rgb00offset = (y * src_width) + x;
            let rgb10offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<REVERSE, B, V, Vis>(
                src_ptr,
                rgb00offset,
                rgb10offset,
                max_value,
                &mut visitor,
                x,
                window,
                y,
            );
        }

        for x in (vectorized_pixels_per_row..window.width).step_by(2) {
            let x = window.x + x;

            let rgb00offset = (y * src_width) + x;
            let rgb10offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<REVERSE, B, f32, Vis>(
                src_ptr,
                rgb00offset,
                rgb10offset,
                max_value,
                &mut visitor,
                x,
                window,
                y,
            );
        }
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn load_and_visit_block<const REVERSE: bool, B, V, Vis>(
    src_ptr: *const B::Primitive,
    rgb00offset: usize,
    rgb10offset: usize,
    max_value: f32,
    visitor: &mut Vis,
    x: usize,
    window: Rect,
    y: usize,
) where
    B: BitsInternal,
    V: Vector,
    Vis: RgbBlockVisitor + RgbBlockVisitorImpl<V>,
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

    let block = RgbBlock {
        rgb00: px00,
        rgb01: px01,
        rgb10: px10,
        rgb11: px11,
    };

    visitor.visit(x - window.x, y - window.y, block);
}
