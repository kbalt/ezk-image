#![allow(clippy::too_many_arguments)]

use super::{I444Block, I444Pixel, I444Visitor, I444VisitorImpl};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{arch::*, max_value_for_bits};
use crate::{PixelFormatPlanes, Rect};

pub(crate) fn read_i444<B, Vis>(
    src_width: usize,
    src_height: usize,
    src_planes: PixelFormatPlanes<&[B::Primitive]>,
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I444Visitor,
{
    assert!(src_planes.bounds_check(src_width, src_height));

    let PixelFormatPlanes::I444 { y, u, v } = src_planes else {
        panic!("Invalid PixelFormatPlanes for read_i444");
    };

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        unsafe {
            return read_i444_avx2::<B, Vis>(
                src_width,
                src_height,
                y,
                u,
                v,
                bits_per_channel,
                window,
                visitor,
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            return read_i444_neon::<B, Vis>(
                src_width,
                src_height,
                y,
                u,
                v,
                bits_per_channel,
                window,
                visitor,
            );
        }
    }

    // Fallback to naive
    unsafe {
        read_i444_impl::<f32, B, Vis>(
            src_width,
            src_height,
            y,
            u,
            v,
            bits_per_channel,
            window,
            visitor,
        )
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[inline(never)]
unsafe fn read_i444_neon<B, Vis>(
    src_width: usize,
    src_height: usize,
    y: &[B::Primitive],
    u: &[B::Primitive],
    v: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I444Visitor,
{
    read_i444_impl::<float32x4_t, B, Vis>(
        src_width,
        src_height,
        y,
        u,
        v,
        bits_per_channel,
        window,
        visitor,
    )
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[inline(never)]
unsafe fn read_i444_avx2<B, Vis>(
    src_width: usize,
    src_height: usize,
    y: &[B::Primitive],
    u: &[B::Primitive],
    v: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I444Visitor,
{
    read_i444_impl::<__m256, B, Vis>(
        src_width,
        src_height,
        y,
        u,
        v,
        bits_per_channel,
        window,
        visitor,
    )
}

#[inline(always)]
unsafe fn read_i444_impl<V, B, Vis>(
    src_width: usize,
    src_height: usize,
    src_y: &[B::Primitive],
    src_u: &[B::Primitive],
    src_v: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    mut visitor: Vis,
) where
    V: Vector,
    B: BitsInternal,
    Vis: I444Visitor + I444VisitorImpl<V>,
{
    let max_value = max_value_for_bits(bits_per_channel);

    let window = window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src_width,
        height: src_height,
    });

    assert!(window.x + window.width <= src_width);
    assert!(window.y + window.height <= src_height);

    assert_eq!(window.width % 2, 0);
    assert_eq!(window.height % 2, 0);

    // How many pixels cannot be vectorized since they don't fit the vector (per row)
    let non_vectored_pixels_per_row = window.width % (V::LEN * 2);
    let vectored_pixels_per_row = window.width - non_vectored_pixels_per_row;

    let y_ptr = src_y.as_ptr();
    let u_ptr = src_u.as_ptr();
    let v_ptr = src_v.as_ptr();

    // Process 2 rows of pixels for iteration of this loop
    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        // Process V::LEN amount of U/V pixel per loop
        // This requires to process V::LEN * 2 Y pixels row since one U/V pixel
        // belongs to 2 Y pixels per row
        for x in (0..vectored_pixels_per_row).step_by(V::LEN * 2) {
            let x = window.x + x;

            let px00_offset = (y * src_width) + x;
            let px10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<V, B, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                u_ptr,
                v_ptr,
                px00_offset,
                px10_offset,
                max_value,
            );
        }

        // Process remaining pixels that couldn't be vectorized
        for x in (0..non_vectored_pixels_per_row).step_by(2) {
            let x = window.x + x + vectored_pixels_per_row;

            let px00_offset = (y * src_width) + x;
            let px10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<f32, B, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                u_ptr,
                v_ptr,
                px00_offset,
                px10_offset,
                max_value,
            );
        }
    }
}

#[inline(always)]
unsafe fn load_and_visit_block<V, B, Vis>(
    visitor: &mut Vis,
    x: usize,
    y: usize,
    y_ptr: *const B::Primitive,
    u_ptr: *const B::Primitive,
    v_ptr: *const B::Primitive,
    px00_offset: usize,
    px10_offset: usize,
    max_value: f32,
) where
    V: Vector,
    Vis: I444VisitorImpl<V>,
    B: BitsInternal,
{
    // Load Y pixels
    let y00 = B::load::<V>(y_ptr.add(px00_offset));
    let y01 = B::load::<V>(y_ptr.add(px00_offset + V::LEN));
    let y10 = B::load::<V>(y_ptr.add(px10_offset));
    let y11 = B::load::<V>(y_ptr.add(px10_offset + V::LEN));

    // Load U pixels
    let u00 = B::load::<V>(u_ptr.add(px00_offset));
    let u01 = B::load::<V>(u_ptr.add(px00_offset + V::LEN));
    let u10 = B::load::<V>(u_ptr.add(px10_offset));
    let u11 = B::load::<V>(u_ptr.add(px10_offset + V::LEN));

    // Load V pixels
    let v00 = B::load::<V>(v_ptr.add(px00_offset));
    let v01 = B::load::<V>(v_ptr.add(px00_offset + V::LEN));
    let v10 = B::load::<V>(v_ptr.add(px10_offset));
    let v11 = B::load::<V>(v_ptr.add(px10_offset + V::LEN));

    // Convert to analog 0..=1.0
    let y00 = y00.vdivf(max_value);
    let y01 = y01.vdivf(max_value);
    let y10 = y10.vdivf(max_value);
    let y11 = y11.vdivf(max_value);

    let u00 = u00.vdivf(max_value);
    let u01 = u01.vdivf(max_value);
    let u10 = u10.vdivf(max_value);
    let u11 = u11.vdivf(max_value);

    let v00 = v00.vdivf(max_value);
    let v01 = v01.vdivf(max_value);
    let v10 = v10.vdivf(max_value);
    let v11 = v11.vdivf(max_value);

    visitor.visit(
        x,
        y,
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
        },
    );
}
