use crate::arch::*;
use crate::{vector::Vector, Rect};

pub(crate) trait ImageReader {
    unsafe fn read_at<V: Vector>(&mut self, x: usize, y: usize);
}

#[inline(never)]
pub(crate) fn read<R>(src_width: usize, src_height: usize, window: Option<Rect>, reader: R)
where
    R: ImageReader,
{
    let window = window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src_width,
        height: src_height,
    });

    assert_eq!(window.width % 2, 0);
    assert_eq!(window.height % 2, 0);

    assert!((window.x + window.width) <= src_width);
    assert!((window.y + window.height) <= src_height);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        #[target_feature(enable = "avx2")]
        unsafe fn call<R>(window: Rect, reader: R)
        where
            R: ImageReader,
        {
            read_impl::<__m256, _>(window, reader);
        }

        // Safety: Did a feature check
        unsafe {
            return call::<R>(window, reader);
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        #[target_feature(enable = "neon")]
        unsafe fn call<R>(window: Rect, reader: R)
        where
            R: ImageReader,
        {
            read_impl::<float32x4_t, _>(window, reader);
        }

        // Safety: Did a feature check
        unsafe {
            return call::<R>(window, reader);
        }
    }

    // Fallback to naive
    // Safety: Inputs have been checked
    unsafe { read_impl::<f32, _>(window, reader) }
}

#[inline(always)]
unsafe fn read_impl<V: Vector, R: ImageReader>(window: Rect, mut reader: R) {
    // How many pixels cannot be vectorized since they don't fit the vector (per row)
    let non_vectored_pixels_per_row = window.width % (V::LEN * 2);
    let vectored_pixels_per_row = window.width - non_vectored_pixels_per_row;

    // Process 2 rows of pixels for iteration of this loop
    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        // Process V::LEN amount of U/V pixel per loop
        // This requires to process V::LEN * 2 Y pixels row since one U/V pixel
        // belongs to 2 Y pixels per row
        for x in (0..vectored_pixels_per_row).step_by(V::LEN * 2) {
            let x = window.x + x;

            reader.read_at::<V>(x, y);
        }

        // Process remaining pixels that couldn't be vectorized
        for x in (0..non_vectored_pixels_per_row).step_by(2) {
            let x = window.x + x + vectored_pixels_per_row;

            reader.read_at::<f32>(x, y);
        }
    }
}
