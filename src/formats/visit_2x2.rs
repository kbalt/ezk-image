use crate::arch::*;
use crate::vector::Vector;

pub(crate) trait Image2x2Visitor {
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize);
}

#[inline(never)]
pub(crate) fn visit<R>(width: usize, height: usize, visitor: R)
where
    R: Image2x2Visitor,
{
    assert_eq!(width % 2, 0);
    assert_eq!(height % 2, 0);

    #[cfg(all(feature = "unstable", any(target_arch = "x86", target_arch = "x86_64")))]
    if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
        #[target_feature(enable = "avx512f", enable = "avx512bw")]
        unsafe fn call<R>(width: usize, height: usize, visitor: R)
        where
            R: Image2x2Visitor,
        {
            visit_impl::<__m512, _>(width, height, visitor);
        }

        // Safety: Did a feature check
        unsafe {
            call::<R>(width, height, visitor);
            return;
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        #[target_feature(enable = "avx2")]
        unsafe fn call<R>(width: usize, height: usize, visitor: R)
        where
            R: Image2x2Visitor,
        {
            visit_impl::<__m256, _>(width, height, visitor);
        }

        // Safety: Did a feature check
        unsafe {
            call::<R>(width, height, visitor);
            return;
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        #[target_feature(enable = "neon")]
        unsafe fn call<R>(width: usize, height: usize, visitor: R)
        where
            R: Image2x2Visitor,
        {
            visit_impl::<float32x4_t, _>(width, height, visitor);
        }

        // Safety: Did a feature check
        unsafe {
            call::<R>(width, height, visitor);
            return;
        }
    }

    // Fallback to naive
    // Safety: Inputs have been checked
    unsafe { visit_impl::<f32, _>(width, height, visitor) };
}

#[inline(always)]
unsafe fn visit_impl<V: Vector, R: Image2x2Visitor>(width: usize, height: usize, mut visitor: R) {
    // How many pixels cannot be vectorized since they don't fit the vector (per row)
    let non_vectored_pixels_per_row = width % (V::LEN * 2);
    let vectored_pixels_per_row = width - non_vectored_pixels_per_row;

    // Process 2 rows of pixels for iteration of this loop
    for y in (0..height).step_by(2) {
        // Process V::LEN * 2 columns per iteration of this loop
        for x in (0..vectored_pixels_per_row).step_by(V::LEN * 2) {
            visitor.visit::<V>(x, y);
        }

        // Process remaining pixels that couldn't be vectorized
        for x in (0..non_vectored_pixels_per_row).step_by(2) {
            let x = x + vectored_pixels_per_row;

            visitor.visit::<f32>(x, y);
        }
    }
}
