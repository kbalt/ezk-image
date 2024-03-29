use super::Vector;
use crate::arch::*;
use crate::color::{ColorOps, ColorOpsPart};
use crate::endian::Endian;
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
        let a = _mm256_and_ps(a, mask);
        let b = _mm256_andnot_ps(mask, b);

        _mm256_or_ps(a, b)
    }
    #[inline(always)]
    unsafe fn vsqrt(self) -> Self {
        _mm256_sqrt_ps(self)
    }

    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn vpow(self, pow: Self) -> Self {
        if cfg!(feature = "veryfastmath") {
            veryfastmath::pow(self, pow)
        } else if cfg!(feature = "fastmath") {
            fastmath::pow(self, pow)
        } else {
            math::pow(self, pow)
        }
    }

    #[target_feature(enable = "avx2", enable = "fma")]
    unsafe fn vln(self) -> Self {
        if cfg!(feature = "veryfastmath") {
            veryfastmath::log(self)
        } else if cfg!(feature = "fastmath") {
            fastmath::log(self)
        } else {
            math::log(self)
        }
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
        let (a, b) = self.zip(other);
        let (a, b) = a.zip(b);
        a.zip(b)
    }

    #[inline(always)]
    unsafe fn load_u8(ptr: *const u8) -> Self {
        let v = ptr.cast::<i64>().read_unaligned();

        let v = _mm256_set1_epi64x(v);

        let lo = _mm256_unpacklo_epi8(v, _mm256_setzero_si256());
        let lo = _mm256_unpacklo_epi8(lo, _mm256_setzero_si256());

        let hi = _mm256_unpackhi_epi8(v, _mm256_setzero_si256());
        let hi = _mm256_unpackhi_epi8(hi, _mm256_setzero_si256());

        let v = _mm256_permute2x128_si256(lo, hi, 0b100000);

        _mm256_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u16<E: Endian>(ptr: *const u16) -> Self {
        let v = ptr.cast::<__m128i>().read_unaligned();

        let v = if E::IS_NATIVE {
            v
        } else {
            util::swap_u16_bytes_x8(v)
        };

        let v = _mm256_set_m128i(v, v);

        let lo = _mm256_unpacklo_epi16(v, _mm256_setzero_si256());
        let hi = _mm256_unpackhi_epi16(v, _mm256_setzero_si256());

        let v = _mm256_permute2x128_si256(lo, hi, 0b100000);

        _mm256_cvtepi32_ps(v)
    }

    #[inline(always)]
    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        #[inline(always)]
        pub(super) unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256) {
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
    unsafe fn load_u16_3x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 3]; 2] {
        #[inline(always)]
        pub(super) unsafe fn inner<E: Endian>(ptr: *const u16) -> (__m256, __m256, __m256) {
            let m1 = __m256::load_u16::<E>(ptr);
            let m2 = __m256::load_u16::<E>(ptr.add(__m256::LEN));
            let m3 = __m256::load_u16::<E>(ptr.add(__m256::LEN * 2));

            deinterleave_3x(m1, m2, m3)
        }

        let (al, bl, cl) = inner::<E>(ptr);
        let (ah, bh, ch) = inner::<E>(ptr.add(Self::LEN * 3));

        [[al, bl, cl], [ah, bh, ch]]
    }

    #[inline(always)]
    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        #[inline(always)]
        pub(super) unsafe fn inner(ptr: *const u8) -> (__m256, __m256, __m256, __m256) {
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
    unsafe fn load_u16_4x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 4]; 2] {
        #[inline(always)]
        pub(super) unsafe fn inner<E: Endian>(ptr: *const u16) -> (__m256, __m256, __m256, __m256) {
            let m1 = __m256::load_u16::<E>(ptr);
            let m2 = __m256::load_u16::<E>(ptr.add(__m256::LEN));
            let m3 = __m256::load_u16::<E>(ptr.add(__m256::LEN * 2));
            let m4 = __m256::load_u16::<E>(ptr.add(__m256::LEN * 3));

            deinterleave_4x(m1, m2, m3, m4)
        }

        let (al, bl, cl, dl) = inner::<E>(ptr);
        let (ah, bh, ch, dh) = inner::<E>(ptr.add(Self::LEN * 4));

        [[al, bl, cl, dl], [ah, bh, ch, dh]]
    }

    #[inline(always)]
    unsafe fn write_u8(self, ptr: *mut u8) {
        ptr.cast::<[u8; 8]>()
            .write_unaligned(util::float32x8_to_u8x8(self))
    }

    #[inline(always)]
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u8; 16]>()
            .write_unaligned(util::float32x8x2_to_u8x16(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_u16<E: Endian>(self, ptr: *mut u16) {
        ptr.cast::<[u16; 8]>()
            .write_unaligned(util::float32x8_to_u16x8::<E>(self))
    }

    #[inline(always)]
    unsafe fn write_u16_2x<E: Endian>(v0: Self, v1: Self, ptr: *mut u16) {
        ptr.cast::<[u16; 16]>()
            .write_unaligned(util::float32x8x2_to_u16x16::<E>(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8) {
        let a = util::pack_f32x8_rgb_u8x24(this[0][0], this[0][1], this[0][2]);
        let b = util::pack_f32x8_rgb_u8x24(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u8; 24]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u16<E: Endian>(this: [[Self; 3]; 2], ptr: *mut u16) {
        let a = util::pack_f32x8_rgb_u16x24::<E>(this[0][0], this[0][1], this[0][2]);
        let b = util::pack_f32x8_rgb_u16x24::<E>(this[1][0], this[1][1], this[1][2]);

        ptr.cast::<[[u16; 24]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8) {
        let a = util::pack_f32x8_rgba_u8x32(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = util::pack_f32x8_rgba_u8x32(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[[u8; 32]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u16<E: Endian>(this: [[Self; 4]; 2], ptr: *mut u16) {
        let a = util::pack_f32x8_rgba_u16x32::<E>(this[0][0], this[0][1], this[0][2], this[0][3]);
        let b = util::pack_f32x8_rgba_u16x32::<E>(this[1][0], this[1][1], this[1][2], this[1][3]);

        ptr.cast::<[[u16; 32]; 2]>().write_unaligned([a, b])
    }

    #[inline(always)]
    fn color_ops(c: &ColorOps) -> &ColorOpsPart<Self> {
        &c.avx2
    }
}

#[inline(always)]
unsafe fn deinterleave_3x(m1: __m256, m2: __m256, m3: __m256) -> (__m256, __m256, __m256) {
    // TODO: write something smarter here, until then - let the compiler figure it out

    let [v00, v01, v02, v03, v04, v05, v06, v07] = transmute::<_, [f32; 8]>(m1);
    let [v08, v09, v10, v11, v12, v13, v14, v15] = transmute::<_, [f32; 8]>(m2);
    let [v16, v17, v18, v19, v20, v21, v22, v23] = transmute::<_, [f32; 8]>(m3);

    let a = transmute([v00, v03, v06, v09, v12, v15, v18, v21]);
    let b = transmute([v01, v04, v07, v10, v13, v16, v19, v22]);
    let c = transmute([v02, v05, v08, v11, v14, v17, v20, v23]);

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

mod math {
    use crate::arch::*;
    use std::f32::consts::LOG2_E;
    use std::mem::transmute;

    // EXP and LOGN functions are copied from https://github.com/reyoung/avx_mathfun
    // found via https://stackoverflow.com/questions/48863719/fastest-implementation-of-exponential-function-using-avx

    const ONE: __m256 = splat(1.0);
    const ONE_HALF: __m256 = splat(0.5);

    const fn splat(f: f32) -> __m256 {
        unsafe { transmute([f; 8]) }
    }

    #[inline(always)]
    pub(super) unsafe fn exp(x: __m256) -> __m256 {
        const L2E: __m256 = splat(LOG2_E); /* log2(e) */
        const L2H: __m256 = splat(-6.931_457_5e-1); /* -log(2)_hi */
        const L2L: __m256 = splat(-1.428_606_8e-6); /* -log(2)_lo */
        /* coefficients for core approximation to exp() in [-log(2)/2, log(2)/2] */
        const C0: __m256 = splat(0.041_944_39);
        const C1: __m256 = splat(0.168_006_67);
        const C2: __m256 = splat(0.499_999_94);
        const C3: __m256 = splat(0.999_956_9);
        const C4: __m256 = splat(0.999_999_64);

        /* exp(x) = 2^i * e^f; i = rint (log2(e) * x), f = x - log(2) * i */
        let t = _mm256_mul_ps(x, L2E); /* t = log2(e) * x */
        let r = _mm256_round_ps(t, _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC); /* r = rint (t) */

        let f = _mm256_fmadd_ps(r, L2H, x); /* x - log(2)_hi * r */
        let f = _mm256_fmadd_ps(r, L2L, f); /* f = x - log(2)_hi * r - log(2)_lo * r */

        let i = _mm256_cvtps_epi32(t); /* i = (int)rint(t) */

        /* p ~= exp (f), -log(2)/2 <= f <= log(2)/2 */
        let p = C0; /* c0 */
        let p = _mm256_fmadd_ps(p, f, C1); /* c0*f+c1 */
        let p = _mm256_fmadd_ps(p, f, C2); /* (c0*f+c1)*f+c2 */
        let p = _mm256_fmadd_ps(p, f, C3); /* ((c0*f+c1)*f+c2)*f+c3 */
        let p = _mm256_fmadd_ps(p, f, C4); /* (((c0*f+c1)*f+c2)*f+c3)*f+c4 ~= exp(f) */

        /* exp(x) = 2^i * p */
        let j = _mm256_slli_epi32(i, 23); /* i << 23 */
        _mm256_castsi256_ps(_mm256_add_epi32(j, _mm256_castps_si256(p))) /* r = p * 2^i */
    }

    #[inline(always)]
    pub(super) unsafe fn log(x: __m256) -> __m256 {
        const MIN_NORM_POS: __m256 = splat(0.0);
        const INV_MANT_MASK: __m256 = splat(unsafe { transmute::<i32, f32>(!0x7f800000) });
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

        let invalid_mask = _mm256_cmp_ps(x, _mm256_setzero_ps(), _CMP_LE_OS);

        let x = _mm256_max_ps(x, MIN_NORM_POS);

        let imm0 = _mm256_srli_epi32(_mm256_castps_si256(x), 23);

        // keep only the fractional part
        let x = _mm256_and_ps(x, INV_MANT_MASK);
        let x = _mm256_or_ps(x, ONE_HALF);

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
        _mm256_or_ps(x, invalid_mask)
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: __m256, y: __m256) -> __m256 {
        exp(_mm256_mul_ps(y, log(x)))
    }
}

pub(crate) mod fastmath {
    // Ported from https://code.google.com/archive/p/fastapprox/
    use crate::arch::*;
    use std::f32::consts::{LN_2, LOG2_E};
    use std::mem::transmute;

    #[inline(always)]
    pub(super) unsafe fn exp(x: __m256) -> __m256 {
        use super::Vector;

        let x = _mm256_mul_ps(x, _mm256_set1_ps(LOG2_E));

        let offset = {
            let mask = _mm256_cmp_ps(x, _mm256_setzero_ps(), _CMP_LT_OQ);

            let zero = _mm256_setzero_ps();
            let one = _mm256_set1_ps(1.0);

            __m256::select(one, zero, mask)
        };

        let clip = _mm256_max_ps(x, _mm256_set1_ps(-126.0));

        let w = _mm256_round_ps(clip, _MM_FROUND_TRUNC);
        let z = _mm256_add_ps(_mm256_sub_ps(clip, w), offset);

        // (1 << 23) *
        //   (clipp + 121.2740575f +
        //     27.7280233f / (4.84252568f - z) - 1.49012907f * z)
        //  )
        let v = _mm256_mul_ps(
            _mm256_set1_ps((1 << 23) as f32),
            _mm256_sub_ps(
                _mm256_add_ps(
                    _mm256_add_ps(clip, _mm256_set1_ps(121.274_055)),
                    _mm256_div_ps(
                        _mm256_set1_ps(27.728_024),
                        _mm256_sub_ps(_mm256_set1_ps(4.842_525_5), z),
                    ),
                ),
                _mm256_mul_ps(z, _mm256_set1_ps(1.490_129_1)),
            ),
        );
        let v = _mm256_cvtps_epi32(v);

        transmute(v)
    }

    #[inline(always)]
    pub(super) unsafe fn log(flt: __m256) -> __m256 {
        let vx_i = transmute::<_, __m256i>(flt);

        let mx_i = _mm256_and_si256(vx_i, _mm256_set1_epi32(0x007FFFFF));
        let mx_i = _mm256_or_si256(mx_i, _mm256_set1_epi32(0x3f000000));
        let mx_f = transmute::<_, __m256>(mx_i);

        let y = _mm256_cvtepi32_ps(vx_i);
        let y = _mm256_mul_ps(y, _mm256_set1_ps(1.192_092_9e-7));

        let y = _mm256_sub_ps(y, _mm256_set1_ps(124.225_52));
        let y = _mm256_sub_ps(y, _mm256_mul_ps(mx_f, _mm256_set1_ps(1.498_030_3)));

        let tmp = _mm256_add_ps(_mm256_set1_ps(0.352_088_72), mx_f);
        let tmp = _mm256_div_ps(_mm256_set1_ps(1.725_88), tmp);
        let y = _mm256_sub_ps(y, tmp);
        _mm256_mul_ps(y, _mm256_set1_ps(LN_2))
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: __m256, y: __m256) -> __m256 {
        exp(_mm256_mul_ps(y, log(x)))
    }
}

pub(crate) mod veryfastmath {
    // Ported from https://code.google.com/archive/p/fastapprox/
    use crate::arch::*;
    use std::f32::consts::LOG2_E;
    use std::mem::transmute;

    #[inline(always)]
    pub(super) unsafe fn exp(x: __m256) -> __m256 {
        let x = _mm256_mul_ps(x, _mm256_set1_ps(LOG2_E));

        let clip = _mm256_max_ps(x, _mm256_set1_ps(-126.0));

        let x = _mm256_cvtps_epi32(_mm256_mul_ps(
            _mm256_set1_ps((1 << 23) as f32),
            _mm256_add_ps(clip, _mm256_set1_ps(126.942_696)),
        ));

        transmute(x)
    }

    #[inline(always)]
    pub(super) unsafe fn log(x: __m256) -> __m256 {
        let x = _mm256_cvtepi32_ps(transmute(x));
        _mm256_fmsub_ps(x, _mm256_set1_ps(8.262_958e-8), _mm256_set1_ps(87.989_97))
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: __m256, y: __m256) -> __m256 {
        exp(_mm256_mul_ps(y, log(x)))
    }
}

pub(crate) mod util {
    use super::*;
    use crate::endian::NativeEndian;

    #[inline(always)]
    pub(crate) unsafe fn float32x8x2_to_u8x16(l: __m256, h: __m256) -> [u8; 16] {
        let l = _mm256_cvtps_epi32(l);
        let h = _mm256_cvtps_epi32(h);

        let v = _mm256_packus_epi32(l, h);
        let v = _mm256_packus_epi16(v, v);

        let v = _mm256_permutevar8x32_epi32(v, _mm256_setr_epi32(0, 4, 3, 5, 0, 4, 3, 5));

        transmute(_mm256_castsi256_si128(v))
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x8x2_to_u16x16<E: Endian>(l: __m256, h: __m256) -> [u16; 16] {
        let l = _mm256_cvtps_epi32(l);
        let h = _mm256_cvtps_epi32(h);

        let v = _mm256_packus_epi32(l, h);
        let v = _mm256_permutevar8x32_epi32(v, _mm256_setr_epi32(0, 1, 4, 5, 2, 3, 6, 7));

        if E::IS_NATIVE {
            transmute(v)
        } else {
            transmute(swap_u16_bytes_x16(v))
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x8_to_u8x8(v: __m256) -> [u8; 8] {
        let v = _mm256_cvtps_epi32(v);
        let v = _mm256_packus_epi32(v, v);
        let v = _mm256_packus_epi16(v, v);

        let a = _mm256_extract_epi32(v, 0);
        let b = _mm256_extract_epi32(v, 4);

        transmute([a, b])
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x8_to_u16x8<E: Endian>(v: __m256) -> [u16; 8] {
        let v = _mm256_cvtps_epi32(v);
        let v = _mm256_packus_epi32(v, v);

        let a = _mm256_extract_epi64(v, 0);
        let b = _mm256_extract_epi64(v, 2);

        let v = _mm_set_epi64x(b, a);

        if E::IS_NATIVE {
            transmute(v)
        } else {
            transmute(swap_u16_bytes_x8(v))
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn pack_f32x8_rgba_u8x32(
        r: __m256,
        g: __m256,
        b: __m256,
        a: __m256,
    ) -> [u8; 32] {
        let [rgba_lo, rgba_hi] = transmute::<[u16; 32], [__m256i; 2]>(pack_f32x8_rgba_u16x32::<
            NativeEndian,
        >(r, g, b, a));

        let rgba = _mm256_packus_epi16(rgba_lo, rgba_hi);

        transmute::<__m256i, [u8; 32]>(rgba)
    }

    #[inline(always)]
    pub(crate) unsafe fn pack_f32x8_rgba_u16x32<E: Endian>(
        r: __m256,
        g: __m256,
        b: __m256,
        a: __m256,
    ) -> [u16; 32] {
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

        let (rgba_lo, rgba_hi) = if E::IS_NATIVE {
            (rgba_lo, rgba_hi)
        } else {
            (
                util::swap_u16_bytes_x16(rgba_lo),
                util::swap_u16_bytes_x16(rgba_hi),
            )
        };

        transmute::<[__m256i; 2], [u16; 32]>([rgba_lo, rgba_hi])
    }

    #[inline(always)]
    pub(crate) unsafe fn pack_f32x8_rgb_u8x24(r: __m256, g: __m256, b: __m256) -> [u8; 24] {
        let rgba = pack_f32x8_rgba_u8x32(r, g, b, _mm256_setzero_ps());

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

        let rgba = _mm256_shuffle_epi8(transmute(rgba), idx);

        // This gets optimized to use avx2 by the compiler
        let [a0, b0, c0, _, a1, b1, c1, _]: [i32; 8] = transmute(rgba);

        transmute([a0, b0, c0, a1, b1, c1])
    }

    #[inline(always)]
    pub(crate) unsafe fn pack_f32x8_rgb_u16x24<E: Endian>(
        r: __m256,
        g: __m256,
        b: __m256,
    ) -> [u16; 24] {
        let rgba = pack_f32x8_rgba_u16x32::<E>(r, g, b, _mm256_setzero_ps());

        let [rgba_lo, rgba_hi] = transmute::<[u16; 32], [__m256i; 2]>(rgba);

        #[rustfmt::skip]
        let idx = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5,
            8, 9, 10, 11, 12, 13,
            16, 17, 18, 19, 20, 21,
            24, 25, 26, 27, 28, 29,
            -128,-128,-128,-128,-128,-128,-128,-128,
        );

        let rgba_lo = _mm256_shuffle_epi8(rgba_lo, idx);
        let rgba_hi = _mm256_shuffle_epi8(rgba_hi, idx);

        // This gets optimized to use avx2 by the compiler
        let [a0, b0, c0, a1, b1, c1, _, _]: [i32; 8] = transmute(rgba_lo);
        let [a2, b2, c2, a3, b3, c3, _, _]: [i32; 8] = transmute(rgba_hi);

        transmute([a0, b0, c0, a1, b1, c1, a2, b2, c2, a3, b3, c3])
    }

    #[inline(always)]
    pub(crate) unsafe fn swap_u16_bytes_x8(a: __m128i) -> __m128i {
        let b = _mm_setr_epi8(1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14);
        _mm_shuffle_epi8(a, b)
    }

    #[inline(always)]
    pub(crate) unsafe fn swap_u16_bytes_x16(a: __m256i) -> __m256i {
        let b = _mm256_setr_epi8(
            1, 0, 3, 2, 5, 4, 7, 6, 9, 8, 11, 10, 13, 12, 15, 14, 17, 16, 19, 18, 21, 20, 23, 22,
            25, 24, 27, 26, 29, 28, 31, 30,
        );
        _mm256_shuffle_epi8(a, b)
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

            assert_eq!(transmute::<_, [f32; 8]>(a), expected_a);
            assert_eq!(transmute::<_, [f32; 8]>(b), expected_b);
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

            assert_eq!(transmute::<_, [f32; 8]>(x), result);
        }
    }

    #[track_caller]
    fn assert_within_error(a: f32, b: f32, error: f32) {
        let err = (a - b).abs();

        assert!(err < error, "error ({err}) too large between {a} and {b}")
    }

    const INPUT: [f32; 8] = [0.1, 0.223, 0.775, 0.5, 2.5, 3.33, 1.1, 0.01];

    #[test]
    fn exp() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            let input: __m256 = transmute(INPUT);

            let math = transmute::<_, [f32; 8]>(math::exp(input));
            let fastmath = transmute::<_, [f32; 8]>(fastmath::exp(input));
            let veryfastmath = transmute::<_, [f32; 8]>(veryfastmath::exp(input));

            let expected = transmute::<_, [f32; 8]>(input).map(|x| x.exp());

            for i in 0..8 {
                println!(
                    "exp({})={} got ({}, {}, {})",
                    INPUT[i], expected[i], math[i], fastmath[i], veryfastmath[i]
                );

                assert_within_error(expected[i], math[i], 0.001);
                assert_within_error(expected[i], fastmath[i], 0.03);
                assert_within_error(expected[i], veryfastmath[i], 0.25);
            }
        }
    }

    #[test]
    fn log() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            let input: __m256 = transmute(INPUT);

            let math = transmute::<_, [f32; 8]>(math::log(input));
            let fastmath = transmute::<_, [f32; 8]>(fastmath::log(input));
            let veryfastmath = transmute::<_, [f32; 8]>(veryfastmath::log(input));

            let expected = transmute::<_, [f32; 8]>(input).map(|x| x.ln());

            for i in 0..8 {
                println!(
                    "ln({})={} got ({}, {}, {})",
                    INPUT[i], expected[i], math[i], fastmath[i], veryfastmath[i]
                );

                assert_within_error(expected[i], math[i], 0.0001);
                assert_within_error(expected[i], fastmath[i], 0.01);
                assert_within_error(expected[i], veryfastmath[i], 0.1);
            }
        }
    }

    #[test]
    fn pow() {
        assert!(is_x86_feature_detected!("avx2"));

        unsafe {
            let input: __m256 = transmute(INPUT);

            let pows = [0.1, 0.2, 1.0, 2.0];

            for pow in pows {
                let math = transmute::<_, [f32; 8]>(math::pow(input, _mm256_set1_ps(pow)));
                let fastmath = transmute::<_, [f32; 8]>(fastmath::pow(input, _mm256_set1_ps(pow)));
                let veryfastmath =
                    transmute::<_, [f32; 8]>(veryfastmath::pow(input, _mm256_set1_ps(pow)));

                let expected = transmute::<_, [f32; 8]>(input).map(|x| x.powf(pow));

                for i in 0..8 {
                    println!(
                        "{}^{pow}={} got ({}, {}, {})",
                        INPUT[i], expected[i], math[i], fastmath[i], veryfastmath[i]
                    );

                    assert_within_error(expected[i], math[i], 0.0001);
                    assert_within_error(expected[i], fastmath[i], 0.01);
                    assert_within_error(expected[i], veryfastmath[i], 00.1);
                }
            }
        }
    }
}
