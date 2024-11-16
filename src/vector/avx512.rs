use super::Vector;
use crate::{arch::*, DynRgbaReader, DynRgbaReaderSpec, RgbaBlock};
use std::mem::transmute;

unsafe impl Vector for __m512 {
    const LEN: usize = 16;
    type Mask = u16;

    #[inline(always)]
    unsafe fn splat(v: f32) -> Self {
        _mm512_set1_ps(v)
    }

    #[inline(always)]
    unsafe fn vadd(self, other: Self) -> Self {
        _mm512_add_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vsub(self, other: Self) -> Self {
        _mm512_sub_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vmul(self, other: Self) -> Self {
        _mm512_mul_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vdiv(self, other: Self) -> Self {
        _mm512_div_ps(self, other)
    }

    #[inline(always)]
    unsafe fn vmax(self, other: Self) -> Self {
        _mm512_max_ps(self, other)
    }

    #[inline(always)]
    unsafe fn lt(self, other: Self) -> Self::Mask {
        _mm512_cmp_ps_mask(self, other, _CMP_LT_OQ)
    }
    #[inline(always)]
    unsafe fn le(self, other: Self) -> Self::Mask {
        _mm512_cmp_ps_mask(self, other, _CMP_LE_OQ)
    }
    #[inline(always)]
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self {
        _mm512_mask_blend_ps(mask, b, a)
    }
    #[inline(always)]
    unsafe fn vsqrt(self) -> Self {
        _mm512_sqrt_ps(self)
    }

    #[target_feature(enable = "avx512f")]
    unsafe fn vpow(self, pow: Self) -> Self {
        math::pow(self, pow)
    }

    #[target_feature(enable = "avx512f")]
    unsafe fn vln(self) -> Self {
        math::log(self)
    }

    #[inline(always)]
    unsafe fn zip(self, other: Self) -> (Self, Self) {
        let i = _mm512_setr_epi32(0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23);
        let a = _mm512_permutex2var_ps(self, i, other);

        let i = _mm512_setr_epi32(8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31);
        let b = _mm512_permutex2var_ps(self, i, other);

        (a, b)
    }

    #[inline(always)]
    unsafe fn unzip(self, other: Self) -> (Self, Self) {
        let i = _mm512_setr_epi32(0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30);
        let a = _mm512_permutex2var_ps(self, i, other);

        let i = _mm512_setr_epi32(1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31);
        let b = _mm512_permutex2var_ps(self, i, other);

        (a, b)
    }

    #[inline(always)]
    unsafe fn load_u8(ptr: *const u8) -> Self {
        let v = ptr.cast::<__m128i>().read_unaligned();
        let v = _mm512_cvtepu8_epi32(v);
        _mm512_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u16(ptr: *const u8) -> Self {
        let v = ptr.cast::<__m256i>().read_unaligned();
        let v = _mm512_cvtepu16_epi32(v);
        _mm512_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m512, __m512, __m512) {
            let m1 = __m512::load_u8(ptr);
            let m2 = __m512::load_u8(ptr.add(__m512::LEN));
            let m3 = __m512::load_u8(ptr.add(__m512::LEN * 2));

            deinterleave_3x(m1, m2, m3)
        }

        let (al, bl, cl) = inner(ptr);
        let (ah, bh, ch) = inner(ptr.add(Self::LEN * 3));

        [[al, bl, cl], [ah, bh, ch]]
    }

    #[inline(always)]
    unsafe fn load_u16_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m512, __m512, __m512) {
            let m1 = __m512::load_u16(ptr);
            let m2 = __m512::load_u16(ptr.add(__m512::LEN));
            let m3 = __m512::load_u16(ptr.add(__m512::LEN * 2));

            deinterleave_3x(m1, m2, m3)
        }

        let (al, bl, cl) = inner(ptr);
        let (ah, bh, ch) = inner(ptr.add(Self::LEN * 3));

        [[al, bl, cl], [ah, bh, ch]]
    }

    #[inline(always)]
    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m512, __m512, __m512, __m512) {
            let m1 = __m512::load_u8(ptr);
            let m2 = __m512::load_u8(ptr.add(__m512::LEN));
            let m3 = __m512::load_u8(ptr.add(__m512::LEN * 2));
            let m4 = __m512::load_u8(ptr.add(__m512::LEN * 3));

            deinterleave_4x(m1, m2, m3, m4)
        }

        let (al, bl, cl, dl) = inner(ptr);
        let (ah, bh, ch, dh) = inner(ptr.add(Self::LEN * 4));

        [[al, bl, cl, dl], [ah, bh, ch, dh]]
    }

    #[inline(always)]
    unsafe fn load_u16_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        #[inline(always)]
        unsafe fn inner(ptr: *const u8) -> (__m512, __m512, __m512, __m512) {
            let m1 = __m512::load_u16(ptr);
            let m2 = __m512::load_u16(ptr.add(__m512::LEN));
            let m3 = __m512::load_u16(ptr.add(__m512::LEN * 2));
            let m4 = __m512::load_u16(ptr.add(__m512::LEN * 3));

            deinterleave_4x(m1, m2, m3, m4)
        }

        let (al, bl, cl, dl) = inner(ptr);
        let (ah, bh, ch, dh) = inner(ptr.add(Self::LEN * 4));

        [[al, bl, cl, dl], [ah, bh, ch, dh]]
    }

    #[inline(always)]
    unsafe fn write_u8(self, ptr: *mut u8) {
        ptr.cast::<[u8; 16]>()
            .write_unaligned(f32x16_to_u8x16(self))
    }

    #[inline(always)]
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u8; 32]>()
            .write_unaligned(f32x16x2_to_u8x32(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_u16(self, ptr: *mut u8) {
        ptr.cast::<[u16; 16]>()
            .write_unaligned(f32x16_to_u16x16(self))
    }

    #[inline(always)]
    unsafe fn write_u16_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u16; 32]>()
            .write_unaligned(f32x16x2_to_u16x32(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8) {
        let a = interleave_f32x16x3_to_u8x48(this[0][0], this[0][1], this[0][2]);
        let b = interleave_f32x16x3_to_u8x48(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u8; 48]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u16(this: [[Self; 3]; 2], ptr: *mut u8) {
        let a = interleave_f32x16x3_to_u16x48(this[0][0], this[0][1], this[0][2]);
        let b = interleave_f32x16x3_to_u16x48(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u16; 48]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8) {
        let a = interleave_f32x16x4_to_u8x64(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = interleave_f32x16x4_to_u8x64(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[__m512i; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u16(this: [[Self; 4]; 2], ptr: *mut u8) {
        let a = interleave_f32x16x4_to_u16x64(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = interleave_f32x16x4_to_u16x64(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[[__m512i; 2]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn dyn_rgba_read<'a>(
        v: &mut (dyn DynRgbaReader + 'a),
        x: usize,
        y: usize,
    ) -> RgbaBlock<Self> {
        DynRgbaReaderSpec::<__m512>::dyn_read(v, x, y)
    }
}

#[inline(always)]
unsafe fn deinterleave_3x(m1: __m512, m2: __m512, m3: __m512) -> (__m512, __m512, __m512) {
    // Red
    let r = {
        let ri = _mm512_setr_epi32(0, 3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 0, 0, 0, 0, 0);
        let r = _mm512_permutex2var_ps(m1, ri, m2);

        let ri = _mm512_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 17, 20, 23, 26, 29);
        _mm512_permutex2var_ps(r, ri, m3)
    };

    // Green
    let g = {
        let gi = _mm512_setr_epi32(1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 0, 0, 0, 0, 0);
        let g = _mm512_permutex2var_ps(m1, gi, m2);

        let gi = _mm512_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 18, 21, 24, 27, 30);
        _mm512_permutex2var_ps(g, gi, m3)
    };

    // Blue
    let b = {
        let bi = _mm512_setr_epi32(2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 0, 0, 0, 0, 0, 0);
        let b = _mm512_permutex2var_ps(m1, bi, m2);

        let bi = _mm512_setr_epi32(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 16, 19, 22, 25, 28, 31);
        _mm512_permutex2var_ps(b, bi, m3)
    };

    (r, g, b)
}

#[inline(always)]
unsafe fn deinterleave_4x(
    m1: __m512,
    m2: __m512,
    m3: __m512,
    m4: __m512,
) -> (__m512, __m512, __m512, __m512) {
    let v1 = _mm512_unpacklo_ps(m1, m2);
    let v2 = _mm512_unpacklo_ps(m3, m4);
    let v3 = _mm512_unpackhi_ps(m1, m2);
    let v4 = _mm512_unpackhi_ps(m3, m4);

    let r = _mm512_unpacklo_ps(v1, v2);
    let g = _mm512_unpackhi_ps(v1, v2);
    let b = _mm512_unpacklo_ps(v3, v4);
    let a = _mm512_unpackhi_ps(v3, v4);

    let idx = _mm512_setr_epi32(0, 4, 8, 12, 2, 6, 10, 14, 1, 5, 9, 13, 3, 7, 11, 15);

    let r = _mm512_permutexvar_ps(idx, r);
    let g = _mm512_permutexvar_ps(idx, g);
    let b = _mm512_permutexvar_ps(idx, b);
    let a = _mm512_permutexvar_ps(idx, a);

    (r, g, b, a)
}

#[inline(always)]
pub(crate) unsafe fn f32x16x2_to_u8x32(l: __m512, h: __m512) -> [u8; 32] {
    let l = _mm512_cvtps_epi32(l);
    let h = _mm512_cvtps_epi32(h);

    let l = _mm512_cvtepi32_epi8(l);
    let h = _mm512_cvtepi32_epi8(h);

    transmute([l, h])
}

#[inline(always)]
pub(crate) unsafe fn f32x16x2_to_u16x32(l: __m512, h: __m512) -> [u16; 32] {
    let l = _mm512_cvtps_epi32(l);
    let h = _mm512_cvtps_epi32(h);

    let l = _mm512_cvtepi32_epi16(l);
    let h = _mm512_cvtepi32_epi16(h);

    transmute([l, h])
}

#[inline(always)]
pub(crate) unsafe fn f32x16_to_u8x16(v: __m512) -> [u8; 16] {
    transmute(_mm512_cvtepi32_epi8(_mm512_cvtps_epi32(v)))
}

#[inline(always)]
pub(crate) unsafe fn f32x16_to_u16x16(v: __m512) -> [u16; 16] {
    transmute(_mm512_cvtepi32_epi16(_mm512_cvtps_epi32(v)))
}

#[inline(always)]
pub(crate) unsafe fn interleave_f32x16x4_to_u8x64(
    r: __m512,
    g: __m512,
    b: __m512,
    a: __m512,
) -> __m512i {
    let [rgba_lo, rgba_hi] = interleave_f32x16x4_to_u16x64(r, g, b, a);

    _mm512_packus_epi16(rgba_lo, rgba_hi)
}

#[inline(always)]
pub(crate) unsafe fn interleave_f32x16x4_to_u16x64(
    r: __m512,
    g: __m512,
    b: __m512,
    a: __m512,
) -> [__m512i; 2] {
    let r = _mm512_cvtps_epi32(r);
    let g = _mm512_cvtps_epi32(g);
    let b = _mm512_cvtps_epi32(b);
    let a = _mm512_cvtps_epi32(a);

    let rb = _mm512_packus_epi32(r, b);
    let ga = _mm512_packus_epi32(g, a);

    let rgba_lo = _mm512_unpacklo_epi16(rb, ga);
    let rgba_hi = _mm512_unpackhi_epi16(rb, ga);

    let (rgba_lo, rgba_hi) = (
        _mm512_unpacklo_epi32(rgba_lo, rgba_hi),
        _mm512_unpackhi_epi32(rgba_lo, rgba_hi),
    );

    [rgba_lo, rgba_hi]
}

#[inline(always)]
pub(crate) unsafe fn interleave_f32x16x3_to_u8x48(r: __m512, g: __m512, b: __m512) -> [u8; 48] {
    let rgb = interleave_f32x16x4_to_u8x64(r, g, b, _mm512_setzero_ps());

    #[rustfmt::skip]
    let idx = _mm512_set_epi8(
        -128 , -128 , -128 , -128,
        14, 13, 12,
        10, 9, 8,
        6, 5, 4,
        2, 1, 0,
        -128 , -128 , -128 , -128,
        14, 13, 12,
        10, 9, 8,
        6, 5, 4,
        2, 1, 0,
        -128 , -128 , -128 , -128,
        14, 13, 12,
        10, 9, 8,
        6, 5, 4,
        2, 1, 0,
        -128 , -128 , -128 , -128,
        14, 13, 12,
        10, 9, 8,
        6, 5, 4,
        2, 1, 0,
    );

    let rgb = _mm512_shuffle_epi8(rgb, idx);

    let [a0, b0, c0, _, a1, b1, c1, _, a2, b2, c2, _, a3, b3, c3, _]: [i32; 16] = transmute(rgb);

    transmute([a0, b0, c0, a1, b1, c1, a2, b2, c2, a3, b3, c3])
}

#[inline(always)]
pub(crate) unsafe fn interleave_f32x16x3_to_u16x48(r: __m512, g: __m512, b: __m512) -> [u16; 48] {
    let [rgb_lo, rgb_hi] = interleave_f32x16x4_to_u16x64(r, g, b, _mm512_setzero_ps());

    #[rustfmt::skip]
    let idx = _mm512_set_epi8(
        -128, -128, -128, -128, -128, -128, -128, -128,
        29, 28, 27, 26, 25, 24, 21, 20, 19, 18, 17,
        16, 13, 12, 11, 10, 9, 8, 5, 4, 3, 2, 1, 0,
        -128, -128, -128, -128, -128, -128, -128, -128,
        29, 28, 27, 26, 25, 24, 21, 20, 19, 18, 17,
        16, 13, 12, 11, 10, 9, 8, 5, 4, 3, 2, 1, 0,
    );

    let rgb_lo = _mm512_shuffle_epi8(rgb_lo, idx);
    let rgb_hi = _mm512_shuffle_epi8(rgb_hi, idx);

    let [a0, b0, c0, a1, b1, c1, _, _, a2, b2, c2, a3, b3, c3, _, _]: [i32; 16] = transmute(rgb_lo);
    let [a4, b4, c4, a5, b5, c5, _, _, a6, b6, c6, a7, b7, c7, _, _]: [i32; 16] = transmute(rgb_hi);

    transmute([
        a0, b0, c0, a1, b1, c1, a2, b2, c2, a3, b3, c3, a4, b4, c4, a5, b5, c5, a6, b6, c6, a7, b7,
        c7,
    ])
}

mod math {
    use crate::arch::*;
    use crate::vector::Vector;
    use std::f32::consts::LOG2_E;
    use std::mem::transmute;

    // EXP and LOGN functions are copied from https://github.com/reyoung/avx_mathfun
    // found via https://stackoverflow.com/questions/48863719/fastest-implementation-of-exponential-function-using-avx

    const ONE: __m512 = splat(1.0);
    const ONE_HALF: __m512 = splat(0.5);

    const fn splat(f: f32) -> __m512 {
        unsafe { transmute::<[f32; 16], __m512>([f; 16]) }
    }

    #[inline(always)]
    pub(super) unsafe fn exp(x: __m512) -> __m512 {
        const EXP_HI: __m512 = splat(88.376_26);
        const EXP_LO: __m512 = splat(-88.376_26);

        const L2E: __m512 = splat(LOG2_E);
        const C1: __m512 = splat(0.693_359_4);
        const C2: __m512 = splat(-2.121_944_4e-4);

        const P0: __m512 = splat(1.987_569_1E-4);
        const P1: __m512 = splat(1.398_199_9E-3);
        const P2: __m512 = splat(8.333_452E-3);
        const P3: __m512 = splat(4.166_579_6E-2);
        const P4: __m512 = splat(1.666_666_6E-1);
        const P5: __m512 = splat(5E-1);

        let x = _mm512_min_ps(EXP_HI, x);
        let x = _mm512_max_ps(EXP_LO, x);

        /* express exp(x) as exp(g + n*log(2)) */
        let fx = _mm512_mul_ps(x, L2E);
        let fx = _mm512_add_ps(fx, ONE_HALF);
        let tmp = _mm512_roundscale_ps(fx, _MM_FROUND_TO_NEG_INF);
        let mask = _mm512_cmp_ps_mask(tmp, fx, _CMP_GT_OS);
        let mask = _mm512_maskz_expand_ps(mask, ONE);
        let fx = _mm512_sub_ps(tmp, mask);
        let tmp = _mm512_mul_ps(fx, C1);
        let z = _mm512_mul_ps(fx, C2);
        let x = _mm512_sub_ps(x, tmp);
        let x = _mm512_sub_ps(x, z);
        let z = _mm512_mul_ps(x, x);

        let y = P0;
        let y = _mm512_fmadd_ps(y, x, P1);
        let y = _mm512_fmadd_ps(y, x, P2);
        let y = _mm512_fmadd_ps(y, x, P3);
        let y = _mm512_fmadd_ps(y, x, P4);
        let y = _mm512_fmadd_ps(y, x, P5);
        let y = _mm512_fmadd_ps(y, z, x);

        let y = _mm512_add_ps(y, ONE);

        /* build 2^n */
        let imm0 = _mm512_cvttps_epi32(fx);
        let imm0 = _mm512_add_epi32(imm0, _mm512_set1_epi32(0x7f));
        let imm0 = _mm512_slli_epi32(imm0, 23);
        let pow2n = _mm512_castsi512_ps(imm0);
        _mm512_mul_ps(y, pow2n)
    }

    #[inline(always)]
    pub(super) unsafe fn log(x: __m512) -> __m512 {
        const INV_MANT_MASK: __m512 = splat(f32::from_bits(!0x7f800000));
        const CEPHES_SQRT_HF: __m512 = splat(0.707_106_77);
        const CEPHES_LOG_P0: __m512 = splat(7.037_683_6E-2);
        const CEPHES_LOG_P1: __m512 = splat(-1.151_461E-1);
        const CEPHES_LOG_P2: __m512 = splat(1.167_699_84E-1);
        const CEPHES_LOG_P3: __m512 = splat(-1.242_014_1E-1);
        const CEPHES_LOG_P4: __m512 = splat(1.424_932_3E-1);
        const CEPHES_LOG_P5: __m512 = splat(-1.666_805_7E-1);
        const CEPHES_LOG_P6: __m512 = splat(2.000_071_4E-1);
        const CEPHES_LOG_P7: __m512 = splat(-2.499_999_4E-1);
        const CEPHES_LOG_P8: __m512 = splat(3.333_333E-1);
        const CEPHES_LOG_Q1: __m512 = splat(-2.121_944_4e-4);
        const CEPHES_LOG_Q2: __m512 = splat(0.693_359_4);

        // Find any numbers lower than 0 or NaN and make a mask for it
        let nan_mask = _mm512_cmp_ps_mask(x, _mm512_setzero_ps(), _CMP_NGE_UQ);
        let x = _mm512_max_ps(_mm512_set1_ps(0.0), x);

        let imm0 = _mm512_srli_epi32(_mm512_castps_si512(x), 23);

        // keep only the fractional part
        let x = _mm512_and_si512(_mm512_castps_si512(INV_MANT_MASK), _mm512_castps_si512(x));
        let x = _mm512_castsi512_ps(_mm512_or_si512(_mm512_castps_si512(ONE_HALF), x));

        let imm0 = _mm512_sub_epi32(imm0, _mm512_set1_epi32(0x7F));
        let e = _mm512_cvtepi32_ps(imm0);

        let e = _mm512_add_ps(e, ONE);

        let mask = _mm512_cmp_ps_mask(x, CEPHES_SQRT_HF, _CMP_LT_OS);
        let tmp = _mm512_mask_blend_ps(mask, _mm512_setzero_ps(), x);

        let x = _mm512_sub_ps(x, ONE);
        let e = _mm512_sub_ps(e, _mm512_mask_blend_ps(mask, _mm512_setzero_ps(), ONE));
        let x = _mm512_add_ps(x, tmp);

        let z = _mm512_mul_ps(x, x);

        let y = CEPHES_LOG_P0;
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P1);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P2);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P3);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P4);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P5);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P6);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P7);
        let y = _mm512_fmadd_ps(y, x, CEPHES_LOG_P8);
        let y = _mm512_mul_ps(y, x);

        let y = _mm512_mul_ps(y, z);

        let tmp = _mm512_mul_ps(e, CEPHES_LOG_Q1);
        let y = _mm512_add_ps(y, tmp);

        let tmp = _mm512_mul_ps(z, ONE_HALF);
        let y = _mm512_sub_ps(y, tmp);

        let tmp = _mm512_mul_ps(e, CEPHES_LOG_Q2);
        let x = _mm512_add_ps(x, y);
        let x = _mm512_add_ps(x, tmp);

        // Any input < 0 and NaN should return NaN
        __m512::select(_mm512_set1_ps(f32::NAN), x, nan_mask)
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: __m512, y: __m512) -> __m512 {
        exp(_mm512_mul_ps(y, log(x)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip() {
        assert!(is_x86_feature_detected!("avx512f"));

        unsafe {
            #[rustfmt::skip]
            let a: [u8; 16] = [
                100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
            ];

            let b: [u8; 16] = [
                200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215,
            ];

            let expected_a = [
                100.0, 200.0, 101.0, 201.0, 102.0, 202.0, 103.0, 203.0, 104.0, 204.0, 105.0, 205.0,
                106.0, 206.0, 107.0, 207.0,
            ];
            let expected_b = [
                108.0, 208.0, 109.0, 209.0, 110.0, 210.0, 111.0, 211.0, 112.0, 212.0, 113.0, 213.0,
                114.0, 214.0, 115.0, 215.0,
            ];

            let a = __m512::load_u8(a.as_ptr());
            let b = __m512::load_u8(b.as_ptr());

            let (a, b) = a.zip(b);

            assert_eq!(transmute::<__m512, [f32; 16]>(a), expected_a);
            assert_eq!(transmute::<__m512, [f32; 16]>(b), expected_b);
        }
    }

    #[test]
    fn unzip() {
        assert!(is_x86_feature_detected!("avx512f"));

        unsafe {
            #[rustfmt::skip]
            let a = [
                100, 101, 102, 103, 104, 105, 106, 107,  108, 109, 110, 111, 112, 113, 114, 115,
            ];

            #[rustfmt::skip]
            let b = [
                200, 201, 202, 203, 204, 205, 206, 207,  208, 209, 210, 211, 212, 213, 214, 215,
            ];

            #[rustfmt::skip]
            let expected_a: [f32; 16] = [
                100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 112.0, 114.0, 200.0, 202.0, 204.0, 206.0, 208.0, 210.0, 212.0, 214.0
            ];

            #[rustfmt::skip]
            let expected_b: [f32; 16] = [
                101.0, 103.0, 105.0, 107.0, 109.0, 111.0, 113.0, 115.0, 201.0, 203.0, 205.0, 207.0, 209.0, 211.0, 213.0, 215.0
            ];

            let a = __m512::load_u8(a.as_ptr());
            let b = __m512::load_u8(b.as_ptr());

            let (a, b) = a.unzip(b);

            assert_eq!(transmute::<__m512, [f32; 16]>(a), expected_a);
            assert_eq!(transmute::<__m512, [f32; 16]>(b), expected_b);
        }
    }

    #[track_caller]
    fn assert_within_error(a: f32, b: f32, error: f32) {
        let err = (a - b).abs();

        assert!(err <= error, "error ({err}) too large between {a} and {b}")
    }

    unsafe fn make_arr(i: __m512) -> [f32; 16] {
        transmute(i)
    }

    #[test]
    fn exp() {
        assert!(is_x86_feature_detected!("avx512f"));

        unsafe {
            assert!(make_arr(math::exp(_mm512_set1_ps(f32::NAN)))[0].is_nan());

            for i in 0..10_000 {
                let i = (i - 5000) as f32 / 1000.0;

                let iv = _mm512_set1_ps(i);

                let rv = math::exp(iv);
                let r = i.exp();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn log() {
        assert!(is_x86_feature_detected!("avx512f"));

        unsafe {
            assert!(make_arr(math::log(_mm512_set1_ps(f32::NAN)))[0].is_nan());
            assert!(make_arr(math::log(_mm512_set1_ps(-3.33)))[0].is_nan());

            for i in 1..10_000 {
                let i = (i) as f32 / 10000.0;
                println!("{i}");

                let iv = _mm512_set1_ps(i);

                let rv = math::log(iv);
                let r = i.ln();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn pow() {
        assert!(is_x86_feature_detected!("avx512f"));

        unsafe {
            assert!(make_arr(math::pow(
                _mm512_set1_ps(f32::NAN),
                _mm512_set1_ps(f32::NAN)
            ))[0]
                .is_nan());

            assert!(make_arr(math::pow(_mm512_set1_ps(1.0), _mm512_set1_ps(f32::NAN)))[0].is_nan());
            assert!(make_arr(math::pow(_mm512_set1_ps(f32::NAN), _mm512_set1_ps(1.0)))[0].is_nan());
            assert!(make_arr(math::log(_mm512_set1_ps(-3.33)))[0].is_nan());

            for a in 1..100 {
                for p in 1..1000 {
                    let a = (a) as f32 / 53.1;
                    let p = (p) as f32 / 839.333;
                    println!("{a}^{p}");

                    let rv = math::pow(_mm512_set1_ps(a), _mm512_set1_ps(p));
                    let r = a.powf(p);

                    let rv = make_arr(rv)[0];

                    assert_within_error(r, rv, 0.0001);
                }
            }
        }
    }
}
