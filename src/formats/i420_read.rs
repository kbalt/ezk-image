use super::i420::{I420Block, I420Visitor, I420VisitorImpl};
use crate::arch::*;
use crate::vector::Vector;
use crate::Rect;

pub(crate) fn read_i420<Vis: I420Visitor>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    visitor: Vis,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        unsafe {
            return read_i420_avx(src_width, src_height, src, window, visitor);
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        unsafe {
            return read_i420_neon(src_width, src_height, src, window, visitor);
        }
    }

    // Fallback to naive
    unsafe { read_i420_impl::<f32, Vis>(src_width, src_height, src, window, visitor) }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[inline(never)]
unsafe fn read_i420_neon<Vis: I420Visitor>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    visitor: Vis,
) {
    read_i420_impl::<float32x4_t, Vis>(src_width, src_height, src, window, visitor)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
#[inline(never)]
unsafe fn read_i420_avx<Vis: I420Visitor>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    visitor: Vis,
) {
    read_i420_impl::<__m256, Vis>(src_width, src_height, src, window, visitor)
}

#[inline(always)]
unsafe fn read_i420_impl<Vec, Vis>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    mut visitor: Vis,
) where
    Vec: Vector,
    Vis: I420Visitor + I420VisitorImpl<Vec>,
{
    assert!(src.len() >= ((src_width * src_height * 12).div_ceil(8)));

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
    let non_vectored_pixels_per_row = window.width % (Vec::LEN * 2);
    let vectored_pixels_per_row = window.width - non_vectored_pixels_per_row;

    let n_pixels = src_width * src_height;

    let y_ptr = src.as_ptr();
    let u_ptr = src.as_ptr().add(n_pixels);
    let v_ptr = u_ptr.add(n_pixels / 4);

    // Process 2 rows of pixels for iteration of this loop
    for y in (0..window.height).step_by(2) {
        let y = window.y + y;
        let hy = y / 2;

        // Process V::LEN amount of U/V pixel per loop
        // This requires to process V::LEN * 2 Y pixels row since one U/V pixel
        // belongs to 2 Y pixels per row
        for x in (0..vectored_pixels_per_row).step_by(Vec::LEN * 2) {
            let x = window.x + x;

            let uv_offset = hy * (src_width / 2) + (x / 2);

            let y00_offset = (y * src_width) + x;
            let y10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<Vec, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                u_ptr,
                v_ptr,
                y00_offset,
                y10_offset,
                uv_offset,
            );
        }

        // Process remaining pixels that couldn't be vectorized
        for x in (0..non_vectored_pixels_per_row).step_by(2) {
            let x = window.x + x + vectored_pixels_per_row;

            let uv_offset = hy * (src_width / 2) + (x / 2);

            let y00_offset = (y * src_width) + x;
            let y10_offset = ((y + 1) * src_width) + x;

            load_and_visit_block::<f32, Vis>(
                &mut visitor,
                x - window.x,
                y - window.y,
                y_ptr,
                u_ptr,
                v_ptr,
                y00_offset,
                y10_offset,
                uv_offset,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
unsafe fn load_and_visit_block<V, Vis>(
    visitor: &mut Vis,
    x: usize,
    y: usize,
    y_ptr: *const u8,
    u_ptr: *const u8,
    v_ptr: *const u8,
    y00_offset: usize,
    y10_offset: usize,
    uv_offset: usize,
) where
    V: Vector,
    Vis: I420VisitorImpl<V>,
{
    // Load Y pixels
    let y00 = V::load(y_ptr.add(y00_offset));
    let y01 = V::load(y_ptr.add(y00_offset + V::LEN));
    let y10 = V::load(y_ptr.add(y10_offset));
    let y11 = V::load(y_ptr.add(y10_offset + V::LEN));

    // Load U and V
    let u = V::load(u_ptr.add(uv_offset));
    let v = V::load(v_ptr.add(uv_offset));

    // Convert 8 bit to analog 0..=1.0
    let y00 = y00.vdivf(255.0);
    let y01 = y01.vdivf(255.0);
    let y10 = y10.vdivf(255.0);
    let y11 = y11.vdivf(255.0);

    let u = u.vdivf(255.0);
    let v = v.vdivf(255.0);

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
