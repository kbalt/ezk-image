use super::Vector;
use crate::{arch::*, DynRgbaReader, DynRgbaReaderSpec, RgbaBlock};
use std::mem::transmute;

unsafe impl Vector for __m256 {
    const LEN: usize = 8;
    type Mask = __m256;

    #[inline(always)]
    unsafe fn splat(v: f32) -> Self {
        _mm256_set1_ps(v)
    }

    #[inline(always)]
    unsafe fn vadd(self, other: Self) -> Self {
        _mm256_add_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vsub(self, other: Self) -> Self {
        _mm256_sub_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vmul(self, other: Self) -> Self {
        _mm256_mul_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vdiv(self, other: Self) -> Self {
        _mm256_div_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vmax(self, other: Self) -> Self {
        _mm256_max_ps(self, other)
    }

    #[inline(always)]
    unsafe fn lt(self, other: Self) -> Self::Mask {
        _mm256_cmp_ps(self, other, _CMP_LT_OQ)
    }
    #[inline(always)]
    unsafe fn le(self, other: Self) -> Self::Mask {
        _mm256_cmp_ps(self, other, _CMP_LE_OQ)
    }
    #[inline(always)]
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self {
        _mm256_blendv_ps(b, a, mask)
    }
    #[inline(always)]
    unsafe fn vsqrt(self) -> Self {
        _mm256_sqrt_ps(self)
    }

    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn vpow(self, pow: Self) -> Self {
        math::pow(self, pow)
    }

    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn vln(self) -> Self {
        math::log(self)
    }

    #[inline(always)]
    unsafe fn zip(self, other: Self) -> (Self, Self) {
        let lo = _mm256_unpacklo_ps(self, other);
        let hi = _mm256_unpackhi_ps(self, other);

        (
            _mm256_permute2f128_ps(lo, hi, 0b10_00_00),
            _mm256_permute2f128_ps(lo, hi, 0b11_00_01),
        )
    }

    #[inline(always)]
    unsafe fn unzip(self, other: Self) -> (Self, Self) {
        let lo = _mm256_shuffle_ps(self, other, 0b10_00_10_00);
        let lo = _mm256_permute4x64_epi64(_mm256_castps_si256(lo), 0b11_01_10_00);

        let hi = _mm256_shuffle_ps(self, other, 0b11_01_11_01);
        let hi = _mm256_permute4x64_epi64(_mm256_castps_si256(hi), 0b11_01_10_00);

        (_mm256_castsi256_ps(lo), _mm256_castsi256_ps(hi))
    }

    #[inline(always)]
    unsafe fn load_u8(ptr: *const u8) -> Self {
        let v = ptr.cast::<i64>().read_unaligned();
        let v = _mm256_cvtepu8_epi32(_mm_set1_epi64x(v));
        _mm256_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u16(ptr: *const u8) -> Self {
        let v = ptr.cast::<__m128i>().read_unaligned();
        let v = _mm256_cvtepu16_epi32(v);
        _mm256_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256) {
            let m1 = __m256::load_u8(ptr);
            let m2 = __m256::load_u8(ptr.add(__m256::LEN));
            let m3 = __m256::load_u8(ptr.add(__m256::LEN * 2));

            deinterleave_3x(m1, m2, m3)
        }

        let (al, bl, cl) = inner(ptr);
        let (ah, bh, ch) = inner(ptr.add(Self::LEN * 3));

        [[al, bl, cl], [ah, bh, ch]]
    }

    #[inline(always)]
    unsafe fn load_u16_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256) {
            let m1 = __m256::load_u16(ptr);
            let m2 = __m256::load_u16(ptr.add(__m256::LEN));
            let m3 = __m256::load_u16(ptr.add(__m256::LEN * 2));

            deinterleave_3x(m1, m2, m3)
        }

        let (al, bl, cl) = inner(ptr);
        let (ah, bh, ch) = inner(ptr.add(Self::LEN * 3));

        [[al, bl, cl], [ah, bh, ch]]
    }

    #[inline(always)]
    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256, __m256) {
            let m1 = __m256::load_u8(ptr);
            let m2 = __m256::load_u8(ptr.add(__m256::LEN));
            let m3 = __m256::load_u8(ptr.add(__m256::LEN * 2));
            let m4 = __m256::load_u8(ptr.add(__m256::LEN * 3));

            deinterleave_4x(m1, m2, m3, m4)
        }

        let (al, bl, cl, dl) = inner(ptr);
        let (ah, bh, ch, dh) = inner(ptr.add(Self::LEN * 4));

        [[al, bl, cl, dl], [ah, bh, ch, dh]]
    }

    #[inline(always)]
    unsafe fn load_u16_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256, __m256) {
            let m1 = __m256::load_u16(ptr);
            let m2 = __m256::load_u16(ptr.add(__m256::LEN));
            let m3 = __m256::load_u16(ptr.add(__m256::LEN * 2));
            let m4 = __m256::load_u16(ptr.add(__m256::LEN * 3));

            deinterleave_4x(m1, m2, m3, m4)
        }

        let (al, bl, cl, dl) = inner(ptr);
        let (ah, bh, ch, dh) = inner(ptr.add(Self::LEN * 4));

        [[al, bl, cl, dl], [ah, bh, ch, dh]]
    }

    #[inline(always)]
    unsafe fn write_u8(self, ptr: *mut u8) {
        ptr.cast::<[u8; 8]>().write_unaligned(f32x8_to_u8x8(self))
    }

    #[inline(always)]
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u8; 16]>()
            .write_unaligned(f32x8x2_to_u8x16(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_u16(self, ptr: *mut u8) {
        ptr.cast::<[u16; 8]>().write_unaligned(f32x8_to_u16x8(self))
    }

    #[inline(always)]
    unsafe fn write_u16_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u16; 16]>()
            .write_unaligned(f32x8x2_to_u16x16(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8) {
        let a = interleave_f32x8x3_to_u8x24(this[0][0], this[0][1], this[0][2]);
        let b = interleave_f32x8x3_to_u8x24(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u8; 24]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u16(this: [[Self; 3]; 2], ptr: *mut u8) {
        let a = interleave_f32x8x3_to_u16x24(this[0][0], this[0][1], this[0][2]);
        let b = interleave_f32x8x3_to_u16x24(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u16; 24]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8) {
        let a = interleave_f32x8x4_to_u8x32(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = interleave_f32x8x4_to_u8x32(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[__m256i; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u16(this: [[Self; 4]; 2], ptr: *mut u8) {
        let a = interleave_f32x8x4_to_u16x32(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = interleave_f32x8x4_to_u16x32(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[[__m256i; 2]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn dyn_rgba_read<'a>(
        v: &mut (dyn DynRgbaReader + 'a),
        x: usize,
        y: usize,
    ) -> RgbaBlock<Self> {
        DynRgbaReaderSpec::<__m256>::dyn_read(v, x, y)
    }
}

#[inline(always)]
unsafe fn deinterleave_3x(m1: __m256, m2: __m256, m3: __m256) -> (__m256, __m256, __m256) {
    // This gets auto vectorized. I tried to see if I get better results using std::simd::simd_swizzle!
    // But it generates the same instructions, so this is fine for now.

    let [v00, v01, v02, v03, v04, v05, v06, v07] = transmute::<__m256, [f32; 8]>(m1);
    let [v08, v09, v10, v11, v12, v13, v14, v15] = transmute::<__m256, [f32; 8]>(m2);
    let [v16, v17, v18, v19, v20, v21, v22, v23] = transmute::<__m256, [f32; 8]>(m3);

    let a = transmute::<[f32; 8], __m256>([v00, v03, v06, v09, v12, v15, v18, v21]);
    let b = transmute::<[f32; 8], __m256>([v01, v04, v07, v10, v13, v16, v19, v22]);
    let c = transmute::<[f32; 8], __m256>([v02, v05, v08, v11, v14, v17, v20, v23]);

    (a, b, c)
}

#[inline(always)]
unsafe fn deinterleave_4x(
    m1: __m256,
    m2: __m256,
    m3: __m256,
    m4: __m256,
) -> (__m256, __m256, __m256, __m256) {
    let v1 = _mm256_unpacklo_ps(m1, m2);
    let v2 = _mm256_unpacklo_ps(m3, m4);
    let v3 = _mm256_unpackhi_ps(m1, m2);
    let v4 = _mm256_unpackhi_ps(m3, m4);

    // A B C D vectors, but their order is scrambled
    let a = _mm256_unpacklo_ps(v1, v2);
    let b = _mm256_unpackhi_ps(v1, v2);
    let c = _mm256_unpacklo_ps(v3, v4);
    let d = _mm256_unpackhi_ps(v3, v4);

    // This vector sorts them into the right order. These operations look expensive,
    // but the compiler will take care of this and remove the permutevar for something
    // much more efficient
    let idx = _mm256_set_epi32(7, 3, 5, 1, 6, 2, 4, 0);

    let a = _mm256_permutevar8x32_ps(a, idx);
    let b = _mm256_permutevar8x32_ps(b, idx);
    let c = _mm256_permutevar8x32_ps(c, idx);
    let d = _mm256_permutevar8x32_ps(d, idx);

    (a, b, c, d)
}

#[inline(always)]
unsafe fn f32x8x2_to_u8x16(l: __m256, h: __m256) -> [u8; 16] {
    let l = _mm256_cvtps_epi32(l);
    let h = _mm256_cvtps_epi32(h);

    let v = _mm256_packus_epi32(l, h);
    let v = _mm256_packus_epi16(v, v);

    let v = _mm256_permutevar8x32_epi32(v, _mm256_setr_epi32(0, 4, 3, 5, 0, 4, 3, 5));

    transmute(_mm256_castsi256_si128(v))
}

#[inline(always)]
unsafe fn f32x8x2_to_u16x16(l: __m256, h: __m256) -> [u16; 16] {
    let l = _mm256_cvtps_epi32(l);
    let h = _mm256_cvtps_epi32(h);

    let v = _mm256_packus_epi32(l, h);
    let v = _mm256_permutevar8x32_epi32(v, _mm256_setr_epi32(0, 1, 4, 5, 2, 3, 6, 7));

    transmute(v)
}

#[inline(always)]
unsafe fn f32x8_to_u8x8(v: __m256) -> [u8; 8] {
    let v = _mm256_cvtps_epi32(v);
    let v = _mm256_packus_epi32(v, v);
    let v = _mm256_packus_epi16(v, v);

    let a = _mm256_extract_epi32(v, 0);
    let b = _mm256_extract_epi32(v, 4);

    transmute([a, b])
}

#[inline(always)]
unsafe fn f32x8_to_u16x8(v: __m256) -> [u16; 8] {
    let v = _mm256_cvtps_epi32(v);
    let v = _mm256_packus_epi32(v, v);

    let a = _mm256_extract_epi64(v, 0);
    let b = _mm256_extract_epi64(v, 2);

    let v = _mm_set_epi64x(b, a);

    transmute(v)
}

#[inline(always)]
unsafe fn interleave_f32x8x4_to_u8x32(r: __m256, g: __m256, b: __m256, a: __m256) -> __m256i {
    let [rgba_lo, rgba_hi] = interleave_f32x8x4_to_u16x32(r, g, b, a);

    _mm256_packus_epi16(rgba_lo, rgba_hi)
}

#[inline(always)]
unsafe fn interleave_f32x8x4_to_u16x32(r: __m256, g: __m256, b: __m256, a: __m256) -> [__m256i; 2] {
    let r = _mm256_cvtps_epi32(r);
    let g = _mm256_cvtps_epi32(g);
    let b = _mm256_cvtps_epi32(b);
    let a = _mm256_cvtps_epi32(a);

    let rb = _mm256_packus_epi32(r, b);
    let ga = _mm256_packus_epi32(g, a);

    let rgba_lo = _mm256_unpacklo_epi16(rb, ga);
    let rgba_hi = _mm256_unpackhi_epi16(rb, ga);

    let (rgba_lo, rgba_hi) = (
        _mm256_unpacklo_epi32(rgba_lo, rgba_hi),
        _mm256_unpackhi_epi32(rgba_lo, rgba_hi),
    );

    [rgba_lo, rgba_hi]
}

#[inline(always)]
unsafe fn interleave_f32x8x3_to_u8x24(r: __m256, g: __m256, b: __m256) -> [u8; 24] {
    let rgb = interleave_f32x8x4_to_u8x32(r, g, b, _mm256_setzero_ps());

    #[rustfmt::skip]
        let idx = _mm256_setr_epi8(
            0, 1, 2,
            4, 5, 6,
            8, 9,10,
            12, 13, 14,
            -128, -128, -128, -128,
            0, 1, 2,
            4, 5, 6,
            8, 9,10,
            12, 13, 14,
            -128, -128, -128, -128,
        );

    let rgb = _mm256_shuffle_epi8(rgb, idx);

    // This gets optimized to use avx2 by the compiler
    let [a0, b0, c0, _, a1, b1, c1, _]: [i32; 8] = transmute(rgb);

    transmute([a0, b0, c0, a1, b1, c1])
}

#[inline(always)]
unsafe fn interleave_f32x8x3_to_u16x24(r: __m256, g: __m256, b: __m256) -> [u16; 24] {
    let [rgb_lo, rgb_hi] = interleave_f32x8x4_to_u16x32(r, g, b, _mm256_setzero_ps());

    #[rustfmt::skip]
        let idx = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5,
            8, 9, 10, 11, 12, 13,
            16, 17, 18, 19, 20, 21,
            24, 25, 26, 27, 28, 29,
            -128,-128,-128,-128,-128,-128,-128,-128,
        );

    let rgb_lo = _mm256_shuffle_epi8(rgb_lo, idx);
    let rgb_hi = _mm256_shuffle_epi8(rgb_hi, idx);

    // This gets optimized to use avx2 by the compiler
    let [a0, b0, c0, a1, b1, c1, _, _]: [i32; 8] = transmute(rgb_lo);
    let [a2, b2, c2, a3, b3, c3, _, _]: [i32; 8] = transmute(rgb_hi);

    transmute([a0, b0, c0, a1, b1, c1, a2, b2, c2, a3, b3, c3])
}

mod math {
    use crate::arch::*;
    use crate::vector::Vector;
    use std::f32::consts::LOG2_E;
    use std::mem::transmute;

    // EXP and LOGN functions are copied from https://github.com/reyoung/avx_mathfun
    // found via https://stackoverflow.com/questions/48863719/fastest-implementation-of-exponential-function-using-avx

    const ONE: __m256 = splat(1.0);
    const ONE_HALF: __m256 = splat(0.5);

    const fn splat(f: f32) -> __m256 {
        unsafe { transmute::<[f32; 8], __m256>([f; 8]) }
    }

    #[inline(always)]
    pub(super) unsafe fn exp(x: __m256) -> __m256 {
        const EXP_HI: __m256 = splat(88.376_26);
        const EXP_LO: __m256 = splat(-88.376_26);

        const L2E: __m256 = splat(LOG2_E);
        const C1: __m256 = splat(0.693_359_4);
        const C2: __m256 = splat(-2.121_944_4e-4);

        const P0: __m256 = splat(1.987_569_1E-4);
        const P1: __m256 = splat(1.398_199_9E-3);
        const P2: __m256 = splat(8.333_452E-3);
        const P3: __m256 = splat(4.166_579_6E-2);
        const P4: __m256 = splat(1.666_666_6E-1);
        const P5: __m256 = splat(5E-1);

        let x = _mm256_min_ps(EXP_HI, x);
        let x = _mm256_max_ps(EXP_LO, x);

        /* express exp(x) as exp(g + n*log(2)) */
        let fx = _mm256_mul_ps(x, L2E);
        let fx = _mm256_add_ps(fx, ONE_HALF);
        let tmp = _mm256_floor_ps(fx);
        let mask = _mm256_cmp_ps(tmp, fx, _CMP_GT_OS);
        let mask = _mm256_and_ps(mask, ONE);
        let fx = _mm256_sub_ps(tmp, mask);
        let tmp = _mm256_mul_ps(fx, C1);
        let z = _mm256_mul_ps(fx, C2);
        let x = _mm256_sub_ps(x, tmp);
        let x = _mm256_sub_ps(x, z);
        let z = _mm256_mul_ps(x, x);

        let y = P0;
        let y = _mm256_fmadd_ps(y, x, P1);
        let y = _mm256_fmadd_ps(y, x, P2);
        let y = _mm256_fmadd_ps(y, x, P3);
        let y = _mm256_fmadd_ps(y, x, P4);
        let y = _mm256_fmadd_ps(y, x, P5);
        let y = _mm256_fmadd_ps(y, z, x);

        let y = _mm256_add_ps(y, ONE);

        /* build 2^n */
        let imm0 = _mm256_cvttps_epi32(fx);
        let imm0 = _mm256_add_epi32(imm0, _mm256_set1_epi32(0x7f));
        let imm0 = _mm256_slli_epi32(imm0, 23);
        let pow2n = _mm256_castsi256_ps(imm0);
        _mm256_mul_ps(y, pow2n)
    }

    #[inline(always)]
    pub(super) unsafe fn log(x: __m256) -> __m256 {
        const INV_MANT_MASK: __m256 = splat(f32::from_bits(!0x7f800000));
        const CEPHES_SQRT_HF: __m256 = splat(0.707_106_77);
        const CEPHES_LOG_P0: __m256 = splat(7.037_683_6E-2);
        const CEPHES_LOG_P1: __m256 = splat(-1.151_461E-1);
        const CEPHES_LOG_P2: __m256 = splat(1.167_699_84E-1);
        const CEPHES_LOG_P3: __m256 = splat(-1.242_014_1E-1);
        const CEPHES_LOG_P4: __m256 = splat(1.424_932_3E-1);
        const CEPHES_LOG_P5: __m256 = splat(-1.666_805_7E-1);
        const CEPHES_LOG_P6: __m256 = splat(2.000_071_4E-1);
        const CEPHES_LOG_P7: __m256 = splat(-2.499_999_4E-1);
        const CEPHES_LOG_P8: __m256 = splat(3.333_333E-1);
        const CEPHES_LOG_Q1: __m256 = splat(-2.121_944_4e-4);
        const CEPHES_LOG_Q2: __m256 = splat(0.693_359_4);

        // Find any numbers lower than 0 or NaN and make a mask for it
        let nan_mask = _mm256_cmp_ps(x, _mm256_setzero_ps(), _CMP_NGE_UQ);
        let x = _mm256_max_ps(_mm256_set1_ps(0.0), x);

        let imm0 = _mm256_srli_epi32(_mm256_castps_si256(x), 23);

        // keep only the fractional part
        let x = _mm256_and_ps(INV_MANT_MASK, x);
        let x = _mm256_or_ps(ONE_HALF, x);

        let imm0 = _mm256_sub_epi32(imm0, _mm256_set1_epi32(0x7F));
        let e = _mm256_cvtepi32_ps(imm0);

        let e = _mm256_add_ps(e, ONE);

        let mask = _mm256_cmp_ps(x, CEPHES_SQRT_HF, _CMP_LT_OS);
        let tmp = _mm256_and_ps(x, mask);

        let x = _mm256_sub_ps(x, ONE);
        let e = _mm256_sub_ps(e, _mm256_and_ps(ONE, mask));
        let x = _mm256_add_ps(x, tmp);

        let z = _mm256_mul_ps(x, x);

        let y = CEPHES_LOG_P0;
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P1);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P2);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P3);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P4);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P5);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P6);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P7);
        let y = _mm256_fmadd_ps(y, x, CEPHES_LOG_P8);
        let y = _mm256_mul_ps(y, x);

        let y = _mm256_mul_ps(y, z);

        let tmp = _mm256_mul_ps(e, CEPHES_LOG_Q1);
        let y = _mm256_add_ps(y, tmp);

        let tmp = _mm256_mul_ps(z, ONE_HALF);
        let y = _mm256_sub_ps(y, tmp);

        let tmp = _mm256_mul_ps(e, CEPHES_LOG_Q2);
        let x = _mm256_add_ps(x, y);
        let x = _mm256_add_ps(x, tmp);

        // Any input < 0 and NaN should return NaN
        __m256::select(_mm256_set1_ps(f32::NAN), x, nan_mask)
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: __m256, y: __m256) -> __m256 {
        exp(_mm256_mul_ps(y, log(x)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            #[rustfmt::skip]
            let rgba = [
                100, 101, 102, 103, 104, 105, 106, 107,
                200, 201, 202, 203, 204, 205, 206, 207,
            ];

            let expected_a = [100.0, 200.0, 101.0, 201.0, 102.0, 202.0, 103.0, 203.0];
            let expected_b = [104.0, 204.0, 105.0, 205.0, 106.0, 206.0, 107.0, 207.0];

            let a = __m256::load_u8(rgba.as_ptr());
            let b = __m256::load_u8(rgba.as_ptr().add(__m256::LEN));

            let (a, b) = a.zip(b);

            assert_eq!(transmute::<__m256, [f32; 8]>(a), expected_a);
            assert_eq!(transmute::<__m256, [f32; 8]>(b), expected_b);
        }
    }

    #[test]
    fn unzip_for_rgb_to_i420_2x2block_add() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            #[rustfmt::skip]
            let red = [
                100, 101, 102, 103, 104, 105, 106, 107,  108, 109, 110, 111, 112, 113, 114, 115,
                200, 201, 202, 203, 204, 205, 206, 207,  208, 209, 210, 211, 212, 213, 214, 215
            ];

            let result = [
                100 + 101 + 200 + 201,
                102 + 103 + 202 + 203,
                104 + 105 + 204 + 205,
                106 + 107 + 206 + 207,
                108 + 109 + 208 + 209,
                110 + 111 + 210 + 211,
                112 + 113 + 212 + 213,
                114 + 115 + 214 + 215,
            ]
            .map(|x| x as f32);

            let r00 = __m256::load_u8(red.as_ptr());
            let r01 = __m256::load_u8(red.as_ptr().add(__m256::LEN));
            let r10 = __m256::load_u8(red.as_ptr().add(__m256::LEN * 2));
            let r11 = __m256::load_u8(red.as_ptr().add(__m256::LEN * 3));

            let r0 = r00.vadd(r10);
            let r1 = r01.vadd(r11);

            let (r0, r1) = r0.unzip(r1);

            let x = r0.vadd(r1);

            assert_eq!(transmute::<__m256, [f32; 8]>(x), result);
        }
    }

    #[track_caller]
    fn assert_within_error(a: f32, b: f32, error: f32) {
        let err = (a - b).abs();

        assert!(err <= error, "error ({err}) too large between {a} and {b}")
    }

    unsafe fn make_arr(i: __m256) -> [f32; 8] {
        transmute(i)
    }

    #[test]
    fn exp() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            assert!(make_arr(math::exp(_mm256_set1_ps(f32::NAN)))[0].is_nan());

            for i in 0..10_000 {
                let i = (i - 5000) as f32 / 1000.0;

                let iv = _mm256_set1_ps(i);

                let rv = math::exp(iv);
                let r = i.exp();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn log() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            assert!(make_arr(math::log(_mm256_set1_ps(f32::NAN)))[0].is_nan());
            assert!(make_arr(math::log(_mm256_set1_ps(-3.33)))[0].is_nan());

            for i in 1..10_000 {
                let i = (i) as f32 / 10000.0;
                println!("{i}");

                let iv = _mm256_set1_ps(i);

                let rv = math::log(iv);
                let r = i.ln();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn pow() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            assert!(make_arr(math::pow(
                _mm256_set1_ps(f32::NAN),
                _mm256_set1_ps(f32::NAN)
            ))[0]
                .is_nan());

            assert!(make_arr(math::pow(_mm256_set1_ps(1.0), _mm256_set1_ps(f32::NAN)))[0].is_nan());
            assert!(make_arr(math::pow(_mm256_set1_ps(f32::NAN), _mm256_set1_ps(1.0)))[0].is_nan());
            assert!(make_arr(math::log(_mm256_set1_ps(-3.33)))[0].is_nan());

            for a in 1..100 {
                for p in 1..1000 {
                    let a = (a) as f32 / 53.1;
                    let p = (p) as f32 / 839.333;
                    println!("{a}^{p}");

                    let rv = math::pow(_mm256_set1_ps(a), _mm256_set1_ps(p));
                    let r = a.powf(p);

                    let rv = make_arr(rv)[0];

                    assert_within_error(r, rv, 0.0001);
                }
            }
        }
    }
}
