use super::rgba::RgbaBlockVisitorImpl;
use crate::arch::*;
use crate::bits::Bits;
use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::vector::Vector;
use crate::Rect;
use std::any::TypeId;

#[inline(never)]
pub(crate) fn read_rgba_4x<const REVERSE: bool, B, Vis>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: Bits,
    Vis: RgbaBlockVisitor,
{
    assert!(src_width * src_height * 4 <= src.len());

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
            src: &[u8],
            bits_per_channel: usize,

            window: Rect,
            visitor: Vis,
        ) where
            B: Bits,
            Vis: RgbaBlockVisitor,
        {
            read_rgba_4x_impl::<REVERSE, B, __m256, _>(
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
            src: &[u8],
            bits_per_channel: usize,
            window: Rect,
            visitor: Vis,
        ) where
            B: Bits,
            Vis: RgbaBlockVisitor,
        {
            read_rgba_4x_impl::<REVERSE, B, float32x4_t, _>(
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
        read_rgba_4x_impl::<REVERSE, B, f32, _>(src_width, src, bits_per_channel, window, visitor)
    }
}

#[inline(always)]
unsafe fn read_rgba_4x_impl<const REVERSE: bool, B, V, Vis>(
    src_width: usize,
    src: &[u8],
    bits_per_channel: usize,
    window: Rect,
    mut visitor: Vis,
) where
    B: Bits,
    V: Vector,
    Vis: RgbaBlockVisitor + RgbaBlockVisitorImpl<V>,
{
    let src_ptr = src.as_ptr().cast::<B::Primitive>();
    let max_value = crate::max_value_for_bits(bits_per_channel);

    let non_vectored_pixels_per_row = window.width % (V::LEN * 2);
    let vectorized_pixels_per_row = window.width - non_vectored_pixels_per_row;

    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        for x in (0..vectorized_pixels_per_row).step_by(V::LEN * 2) {
            let x = window.x + x;

            let rgba00offset = ((y * src_width) + x) * 4;
            let rgba10offset = rgba00offset + (src_width * 4);

            let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
                B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba00offset));
            let [[r10, g10, b10, a10], [r11, g11, b11, a11]] =
                B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba10offset));

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

            let block = RgbaBlock {
                rgba00: px00,
                rgba01: px01,
                rgba10: px10,
                rgba11: px11,
            };

            visitor.visit(x - window.x, y - window.y, block);
        }
    }

    // Recurse only if this function was called with anything else than f32
    if TypeId::of::<V>() != TypeId::of::<f32>() {
        read_rgba_4x_impl::<REVERSE, B, f32, Vis>(
            src_width,
            src,
            bits_per_channel,
            Rect {
                x: window.x + vectorized_pixels_per_row,
                y: window.y,
                width: window.width - vectorized_pixels_per_row,
                height: window.height,
            },
            visitor,
        );
    }
}
