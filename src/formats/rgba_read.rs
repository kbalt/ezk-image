use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::{arch::*, Rect};

#[inline(never)]
pub(crate) fn read_rgba_4x<const REVERSE: bool, Vis: RgbaBlockVisitor>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    visitor: Vis,
) {
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
        // Safety: Did a feature check
        unsafe {
            return avx2::read_rgba_4x::<REVERSE, _>(src_width, src, window, visitor);
        }
    }

    #[cfg(target_arch = "aarch64")]
    if is_aarch64_feature_detected!("neon") {
        // Safety: Did a feature check
        unsafe {
            return neon::read_rgba_4x::<REVERSE, _>(src_width, src, window, visitor);
        }
    }

    // Fallback to naive
    // Safety: Inputs have been checked
    unsafe { read_rgba_4x_naive::<REVERSE, _>(src_width, src, window, visitor) }
}

#[inline(always)]
unsafe fn load_rgba_naive(ptr: *const u8) -> (f32, f32, f32, f32) {
    (
        f32::from(ptr.read_unaligned()),
        f32::from(ptr.add(1).read_unaligned()),
        f32::from(ptr.add(2).read_unaligned()),
        f32::from(ptr.add(3).read_unaligned()),
    )
}

unsafe fn read_rgba_4x_naive<const REVERSE: bool, Vis: RgbaBlockVisitor>(
    src_width: usize,
    src: &[u8],
    window: Rect,
    mut visitor: Vis,
) {
    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        for x in (0..window.width).step_by(2) {
            let x = window.x + x;

            let rgba00offset = ((y * src_width) + x) * 4;
            let rgba10offset = rgba00offset + (src_width * 4);

            let (r00, g00, b00, a00) = load_rgba_naive(src.as_ptr().add(rgba00offset));
            let (r01, g01, b01, a01) = load_rgba_naive(src.as_ptr().add(rgba00offset + 4));
            let (r10, g10, b10, a10) = load_rgba_naive(src.as_ptr().add(rgba10offset));
            let (r11, g11, b11, a11) = load_rgba_naive(src.as_ptr().add(rgba10offset + 4));

            let px00 = RgbaPixel::from_loaded8::<REVERSE>(r00, g00, b00, a00);
            let px01 = RgbaPixel::from_loaded8::<REVERSE>(r01, g01, b01, a01);
            let px10 = RgbaPixel::from_loaded8::<REVERSE>(r10, g10, b10, a10);
            let px11 = RgbaPixel::from_loaded8::<REVERSE>(r11, g11, b11, a11);

            let block = RgbaBlock {
                rgba00: px00,
                rgba01: px01,
                rgba10: px10,
                rgba11: px11,
            };

            visitor.visit(x - window.x, y - window.y, block);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2 {
    use super::*;
    use crate::vector::Vector;

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn load_rgba(ptr: *const u8) -> (__m256, __m256, __m256, __m256) {
        let m1 = __m256::load_u8(ptr);
        let m2 = __m256::load_u8(ptr.add(__m256::LEN));
        let m3 = __m256::load_u8(ptr.add(__m256::LEN * 2));
        let m4 = __m256::load_u8(ptr.add(__m256::LEN * 3));

        let v1 = _mm256_unpacklo_ps(m1, m2);
        let v2 = _mm256_unpacklo_ps(m3, m4);
        let v3 = _mm256_unpackhi_ps(m1, m2);
        let v4 = _mm256_unpackhi_ps(m3, m4);

        // R G B A vectors, but their order is scrambled
        let r = _mm256_unpacklo_ps(v1, v2);
        let g = _mm256_unpackhi_ps(v1, v2);
        let b = _mm256_unpacklo_ps(v3, v4);
        let a = _mm256_unpackhi_ps(v3, v4);

        // This vector sorts them into the right order. These operations look expensive,
        // but the compiler will take care of this and remove the permutevar for something
        // much more efficient
        let idx = _mm256_set_epi32(7, 3, 5, 1, 6, 2, 4, 0);

        let r = _mm256_permutevar8x32_ps(r, idx);
        let g = _mm256_permutevar8x32_ps(g, idx);
        let b = _mm256_permutevar8x32_ps(b, idx);
        let a = _mm256_permutevar8x32_ps(a, idx);

        (r, g, b, a)
    }

    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn read_rgba_4x<const REVERSE: bool, Vis: RgbaBlockVisitor>(
        src_width: usize,
        src: &[u8],
        window: Rect,
        mut visitor: Vis,
    ) {
        let non_vectored_pixels_per_row = window.width % (__m256::LEN * 2);
        let vectorized_pixels_per_row = window.width - non_vectored_pixels_per_row;

        for y in (0..window.height).step_by(2) {
            let y = window.y + y;

            for x in (0..vectorized_pixels_per_row).step_by(__m256::LEN * 2) {
                let x = window.x + x;

                let rgba00offset = ((y * src_width) + x) * 4;
                let rgba10offset = rgba00offset + (src_width * 4);

                let (r00, g00, b00, a00) = load_rgba(src.as_ptr().add(rgba00offset));
                let (r01, g01, b01, a01) =
                    load_rgba(src.as_ptr().add(rgba00offset + (__m256::LEN * 4)));
                let (r10, g10, b10, a10) = load_rgba(src.as_ptr().add(rgba10offset));
                let (r11, g11, b11, a11) =
                    load_rgba(src.as_ptr().add(rgba10offset + (__m256::LEN * 4)));

                let px00 = RgbaPixel::from_loaded8::<REVERSE>(r00, g00, b00, a00);
                let px01 = RgbaPixel::from_loaded8::<REVERSE>(r01, g01, b01, a01);
                let px10 = RgbaPixel::from_loaded8::<REVERSE>(r10, g10, b10, a10);
                let px11 = RgbaPixel::from_loaded8::<REVERSE>(r11, g11, b11, a11);

                let block = RgbaBlock {
                    rgba00: px00,
                    rgba01: px01,
                    rgba10: px10,
                    rgba11: px11,
                };

                visitor.visit(x - window.x, y - window.y, block);
            }
        }

        // Do the remaining pixels
        super::read_rgba_4x_naive::<REVERSE, _>(
            src_width,
            src,
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

#[cfg(target_arch = "aarch64")]
mod neon {
    use super::*;
    use crate::vector::Vector;
    use std::mem::transmute;

    #[inline(always)]
    pub(super) unsafe fn load_rgba(ptr: *const u8) -> [[float32x4_t; 4]; 2] {
        let rgba_lanes = vld4_u8(ptr);

        let [r_lane, g_lane, b_lane, a_lane]: [uint8x8_t; 4] = transmute(rgba_lanes);

        let r = vmovl_u8(r_lane);
        let rl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(r)));
        let rh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(r)));

        let g = vmovl_u8(g_lane);
        let gl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(g)));
        let gh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(g)));

        let b = vmovl_u8(b_lane);
        let bl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(b)));
        let bh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(b)));

        let a = vmovl_u8(a_lane);
        let al = vcvtq_f32_u32(vmovl_u16(vget_low_u16(a)));
        let ah = vcvtq_f32_u32(vmovl_u16(vget_high_u16(a)));

        [[rl, gl, bl, al], [rh, gh, bh, ah]]
    }

    #[target_feature(enable = "neon")]
    pub(crate) unsafe fn read_rgba_4x<const REVERSE: bool, Vis: RgbaBlockVisitor>(
        src_width: usize,
        src: &[u8],
        window: Rect,
        mut visitor: Vis,
    ) {
        let non_vectored_pixels_per_row = window.width % (float32x4_t::LEN * 2);
        let vectorized_pixels_per_row = window.width - non_vectored_pixels_per_row;

        for y in (0..window.height).step_by(2) {
            let y = window.y + y;

            for x in (0..window.width).step_by(float32x4_t::LEN * 2) {
                let x = window.x + x;

                let rgba00offset = ((y * src_width) + x) * 4;
                let rgba10offset = rgba00offset + (src_width * 4);

                let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
                    load_rgba(src.as_ptr().add(rgba00offset));
                let [[r10, g10, b10, a10], [r11, g11, b11, a11]] =
                    load_rgba(src.as_ptr().add(rgba10offset));

                let px00 = RgbaPixel::from_loaded8::<REVERSE>(r00, g00, b00, a00);
                let px01 = RgbaPixel::from_loaded8::<REVERSE>(r01, g01, b01, a01);
                let px10 = RgbaPixel::from_loaded8::<REVERSE>(r10, g10, b10, a10);
                let px11 = RgbaPixel::from_loaded8::<REVERSE>(r11, g11, b11, a11);

                let block = RgbaBlock {
                    rgba00: px00,
                    rgba01: px01,
                    rgba10: px10,
                    rgba11: px11,
                };

                visitor.visit(x - window.x, y - window.y, block);
            }
        }

        // Do the remaining pixels
        super::read_rgba_4x_naive::<REVERSE, _>(
            src_width,
            src,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::transmute;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[test]
    fn test_load_rgba() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            #[rustfmt::skip]
            let rgba = [
                100, 101, 102, 103,
                110, 111, 112, 113,
                120, 121, 122, 123,
                130, 131, 132, 133,
                140, 141, 142, 143,
                150, 151, 152, 153,
                160, 161, 162, 163,
                170, 171, 172, 173,
            ];

            let expected_r = [100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0];
            let expected_g = [101.0, 111.0, 121.0, 131.0, 141.0, 151.0, 161.0, 171.0];
            let expected_b = [102.0, 112.0, 122.0, 132.0, 142.0, 152.0, 162.0, 172.0];
            let expected_a = [103.0, 113.0, 123.0, 133.0, 143.0, 153.0, 163.0, 173.0];

            let (r, g, b, a) = avx2::load_rgba(rgba.as_ptr());

            assert_eq!(transmute::<_, [f32; 8]>(r), expected_r);
            assert_eq!(transmute::<_, [f32; 8]>(g), expected_g);
            assert_eq!(transmute::<_, [f32; 8]>(b), expected_b);
            assert_eq!(transmute::<_, [f32; 8]>(a), expected_a);
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_load_rgba() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            #[rustfmt::skip]
            let rgba = [
                100, 101, 102, 103,
                110, 111, 112, 113,
                120, 121, 122, 123,
                130, 131, 132, 133,
                140, 141, 142, 143,
                150, 151, 152, 153,
                160, 161, 162, 163,
                170, 171, 172, 173,
            ];

            let expected_r0: [f32; 4] = [100.0, 110.0, 120.0, 130.0];
            let expected_r1: [f32; 4] = [140.0, 150.0, 160.0, 170.0];
            let expected_g0: [f32; 4] = [101.0, 111.0, 121.0, 131.0];
            let expected_g1: [f32; 4] = [141.0, 151.0, 161.0, 171.0];
            let expected_b0: [f32; 4] = [102.0, 112.0, 122.0, 132.0];
            let expected_b1: [f32; 4] = [142.0, 152.0, 162.0, 172.0];
            let expected_a0: [f32; 4] = [103.0, 113.0, 123.0, 133.0];
            let expected_a1: [f32; 4] = [143.0, 153.0, 163.0, 173.0];

            let [[r0, g0, b0, a0], [r1, g1, b1, a1]] = load_rgba(rgba.as_ptr());

            assert_eq!(transmute::<_, [f32; 4]>(r0), expected_r0);
            assert_eq!(transmute::<_, [f32; 4]>(r1), expected_r1);

            assert_eq!(transmute::<_, [f32; 4]>(g0), expected_g0);
            assert_eq!(transmute::<_, [f32; 4]>(g1), expected_g1);

            assert_eq!(transmute::<_, [f32; 4]>(b0), expected_b0);
            assert_eq!(transmute::<_, [f32; 4]>(b1), expected_b1);

            assert_eq!(transmute::<_, [f32; 4]>(a0), expected_a0);
            assert_eq!(transmute::<_, [f32; 4]>(a1), expected_a1);
        }
    }
}
