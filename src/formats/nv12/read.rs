#![allow(clippy::too_many_arguments)]

use crate::bits::BitsInternal;
use crate::formats::{I420Block, I420Visitor};
use crate::vector::Vector;
use crate::{arch::*, max_value_for_bits};
use crate::{PixelFormatPlanes, Rect};

pub(crate) fn read_nv12<B, Vis>(
    src_width: usize,
    src_height: usize,
    src_planes: PixelFormatPlanes<&[B::Primitive]>,
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I420Visitor,
{
    assert!(src_planes.bounds_check(src_width, src_height));

    let PixelFormatPlanes::NV12 { y, uv } = src_planes else {
        panic!("Invalid PixelFormatPlanes for read_nv12");
    };

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        unsafe {
            return read_nv12_avx2::<B, Vis>(
                src_width,
                src_height,
                y,
                uv,
                bits_per_channel,
                window,
                visitor,
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            return read_nv12_neon::<B, Vis>(
                src_width,
                src_height,
                y,
                uv,
                bits_per_channel,
                window,
                visitor,
            );
        }
    }

    // Fallback to naive
    unsafe {
        read_nv12_impl::<f32, B, Vis>(
            src_width,
            src_height,
            y,
            uv,
            bits_per_channel,
            window,
            visitor,
        )
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[inline(never)]
unsafe fn read_nv12_neon<B, Vis>(
    src_width: usize,
    src_height: usize,
    y: &[B::Primitive],
    uv: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I420Visitor,
{
    read_nv12_impl::<float32x4_t, B, Vis>(
        src_width,
        src_height,
        y,
        uv,
        bits_per_channel,
        window,
        visitor,
    )
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[inline(never)]
unsafe fn read_nv12_avx2<B, Vis>(
    src_width: usize,
    src_height: usize,
    y: &[B::Primitive],
    uv: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    visitor: Vis,
) where
    B: BitsInternal,
    Vis: I420Visitor,
{
    read_nv12_impl::<__m256, B, Vis>(
        src_width,
        src_height,
        y,
        uv,
        bits_per_channel,
        window,
        visitor,
    )
}

#[inline(always)]
unsafe fn read_nv12_impl<V, B, Vis>(
    src_width: usize,
    src_height: usize,
    src_y: &[B::Primitive],
    src_uv: &[B::Primitive],
    bits_per_channel: usize,
    window: Option<Rect>,
    mut visitor: Vis,
) where
    V: Vector,
    B: BitsInternal,
    Vis: I420Visitor,
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
    let uv_ptr = src_uv.as_ptr();

    // Process 2 rows of pixels for iteration of this loop
    for y in (0..window.height).step_by(2) {
        let y = window.y + y;
        let hy = y / 2;

        // Process V::LEN amount of U/V pixel per loop
        // This requires to process V::LEN * 2 Y pixels row since one U/V pixel
        // belongs to 2 Y pixels per row
        for x in (0..vectored_pixels_per_row).step_by(V::LEN * 2) {
            let x = window.x + x;

            let uv_offset = hy * (src_width / 2) + (x / 2);

            let y00_offset = (y * src_width) + x;
            let y10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<V, B, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                uv_ptr,
                y00_offset,
                y10_offset,
                uv_offset * 2,
                max_value,
            );
        }

        // Process remaining pixels that couldn't be vectorized
        for x in (0..non_vectored_pixels_per_row).step_by(2) {
            let x = window.x + x + vectored_pixels_per_row;

            let uv_offset = hy * (src_width / 2) + (x / 2);

            let y00_offset = (y * src_width) + x;
            let y10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<f32, B, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                uv_ptr,
                y00_offset,
                y10_offset,
                uv_offset * 2,
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
    uv_ptr: *const B::Primitive,
    y00_offset: usize,
    y10_offset: usize,
    uv_offset: usize,
    max_value: f32,
) where
    V: Vector,
    Vis: I420Visitor,
    B: BitsInternal,
{
    // Load Y pixels
    let y00 = B::load::<V>(y_ptr.add(y00_offset));
    let y01 = B::load::<V>(y_ptr.add(y00_offset + V::LEN));
    let y10 = B::load::<V>(y_ptr.add(y10_offset));
    let y11 = B::load::<V>(y_ptr.add(y10_offset + V::LEN));

    // Load U and V
    let uv0 = B::load::<V>(uv_ptr.add(uv_offset));
    let uv1 = B::load::<V>(uv_ptr.add(uv_offset + V::LEN));

    let (u, v) = uv0.unzip(uv1);

    // Convert to analog 0..=1.0
    let y00 = y00.vdivf(max_value);
    let y01 = y01.vdivf(max_value);
    let y10 = y10.vdivf(max_value);
    let y11 = y11.vdivf(max_value);

    let u = u.vdivf(max_value);
    let v = v.vdivf(max_value);

    visitor.visit(
        x,
        y,
        I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        },
    );
}
