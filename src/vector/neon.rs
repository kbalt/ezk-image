use super::Vector;
use crate::arch::*;
use crate::endian::Endian;
use crate::{DynRgbaReader, DynRgbaReaderSpec, RgbaBlock};
use std::mem::transmute;

unsafe impl Vector for float32x4_t {
    const LEN: usize = 4;
    type Mask = uint32x4_t;

    #[inline(always)]
    unsafe fn splat(v: f32) -> Self {
        vdupq_n_f32(v)
    }

    #[inline(always)]
    unsafe fn vadd(self, other: Self) -> Self {
        vaddq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn vsub(self, other: Self) -> Self {
        vsubq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn vmul(self, other: Self) -> Self {
        vmulq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn vdiv(self, other: Self) -> Self {
        vdivq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn vmax(self, other: Self) -> Self {
        vmaxq_f32(self, other)
    }

    #[inline(always)]
    unsafe fn lt(self, other: Self) -> Self::Mask {
        vcaltq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn le(self, other: Self) -> Self::Mask {
        vcaleq_f32(self, other)
    }
    #[inline(always)]
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self {
        vbslq_f32(mask, a, b)
    }

    #[inline(always)]
    unsafe fn vsqrt(self) -> Self {
        vsqrtq_f32(self)
    }

    #[inline(always)]
    unsafe fn vpow(self, pow: Self) -> Self {
        math::pow(self, pow)
    }
    #[inline(always)]
    unsafe fn vln(self) -> Self {
        math::log(self)
    }

    #[inline(always)]
    unsafe fn zip(self, other: Self) -> (Self, Self) {
        let a = vzip1q_f32(self, other);
        let b = vzip2q_f32(self, other);

        (a, b)
    }

    #[inline(always)]
    unsafe fn unzip(self, other: Self) -> (Self, Self) {
        let a = vuzp1q_f32(self, other);
        let b = vuzp2q_f32(self, other);

        (a, b)
    }

    #[inline(always)]
    unsafe fn load_u8(ptr: *const u8) -> Self {
        let v = ptr.cast::<[u8; 4]>().read_unaligned();
        let v = vmovl_u8(transmute([v, v]));
        let v = vmovl_high_u16(v);

        vcvtq_f32_u32(v)
    }

    #[inline(always)]
    unsafe fn load_u16<E: Endian>(ptr: *const u16) -> Self {
        let v = ptr.cast::<uint16x4_t>().read_unaligned();
        let v = if E::IS_NATIVE { v } else { vrev32_u16(v) };
        let v = vmovl_u16(v);

        vcvtq_f32_u32(v)
    }

    #[inline(always)]
    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        let rgb_lanes = vld3_u8(ptr);

        let [r_lane, g_lane, b_lane]: [uint8x8_t; 3] = transmute(rgb_lanes);

        let r = vmovl_u8(r_lane);
        let rl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(r)));
        let rh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(r)));

        let g = vmovl_u8(g_lane);
        let gl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(g)));
        let gh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(g)));

        let b = vmovl_u8(b_lane);
        let bl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(b)));
        let bh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(b)));

        [[rl, gl, bl], [rh, gh, bh]]
    }

    #[inline(always)]
    unsafe fn load_u16_3x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 3]; 2] {
        let rgb_lanes = vld3q_u16(ptr);

        let [r, g, b]: [uint16x8_t; 3] = transmute(rgb_lanes);

        let (r, g, b) = if E::IS_NATIVE {
            (r, g, b)
        } else {
            (vrev32q_u16(r), vrev32q_u16(g), vrev32q_u16(b))
        };

        let rl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(r)));
        let rh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(r)));

        let gl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(g)));
        let gh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(g)));

        let bl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(b)));
        let bh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(b)));

        [[rl, gl, bl], [rh, gh, bh]]
    }

    #[inline(always)]
    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
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

    #[inline(always)]
    unsafe fn load_u16_4x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 4]; 2] {
        let rgba_lanes = vld4q_u16(ptr);

        let [r, g, b, a]: [uint16x8_t; 4] = transmute(rgba_lanes);

        let (r, g, b, a) = if E::IS_NATIVE {
            (r, g, b, a)
        } else {
            (
                vrev32q_u16(r),
                vrev32q_u16(g),
                vrev32q_u16(b),
                vrev32q_u16(a),
            )
        };

        let rl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(r)));
        let rh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(r)));

        let gl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(g)));
        let gh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(g)));

        let bl = vcvtq_f32_u32(vmovl_u16(vget_low_u16(b)));
        let bh = vcvtq_f32_u32(vmovl_u16(vget_high_u16(b)));

        let al = vcvtq_f32_u32(vmovl_u16(vget_low_u16(a)));
        let ah = vcvtq_f32_u32(vmovl_u16(vget_high_u16(a)));

        [[rl, gl, bl, al], [rh, gh, bh, ah]]
    }

    #[inline(always)]
    unsafe fn write_u8(self, ptr: *mut u8) {
        ptr.cast::<[u8; 4]>()
            .write_unaligned(util::float32x4_to_u8x4(self))
    }

    #[inline(always)]
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u8; 8]>()
            .write_unaligned(util::float32x4x2_to_u8x8(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_u16<E: Endian>(self, ptr: *mut u16) {
        ptr.cast::<[u16; 4]>()
            .write_unaligned(util::float32x4_to_u16x4::<E>(self))
    }

    #[inline(always)]
    unsafe fn write_u16_2x<E: Endian>(v0: Self, v1: Self, ptr: *mut u16) {
        ptr.cast::<[u16; 8]>()
            .write_unaligned(util::float32x4x2_to_u16x8::<E>(v0, v1))
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8) {
        let v0 = util::float32x4x2_to_u8x8(this[0][0], this[1][0]);
        let v1 = util::float32x4x2_to_u8x8(this[0][1], this[1][1]);
        let v2 = util::float32x4x2_to_u8x8(this[0][2], this[1][2]);

        let v = transmute::<[[u8; 8]; 3], uint8x8x3_t>([v0, v1, v2]);

        vst3_u8(ptr, v)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u16<E: Endian>(this: [[Self; 3]; 2], ptr: *mut u16) {
        let v0 = util::float32x4x2_to_u16x8::<E>(this[0][0], this[1][0]);
        let v1 = util::float32x4x2_to_u16x8::<E>(this[0][1], this[1][1]);
        let v2 = util::float32x4x2_to_u16x8::<E>(this[0][2], this[1][2]);

        let v = transmute::<[[u16; 8]; 3], uint16x8x3_t>([v0, v1, v2]);

        vst3q_u16(ptr, v)
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8) {
        let v0 = util::float32x4x2_to_u8x8(this[0][0], this[1][0]);
        let v1 = util::float32x4x2_to_u8x8(this[0][1], this[1][1]);
        let v2 = util::float32x4x2_to_u8x8(this[0][2], this[1][2]);
        let v3 = util::float32x4x2_to_u8x8(this[0][3], this[1][3]);

        let v = transmute::<[[u8; 8]; 4], uint8x8x4_t>([v0, v1, v2, v3]);

        vst4_u8(ptr, v)
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u16<E: Endian>(this: [[Self; 4]; 2], ptr: *mut u16) {
        let v0 = util::float32x4x2_to_u16x8::<E>(this[0][0], this[1][0]);
        let v1 = util::float32x4x2_to_u16x8::<E>(this[0][1], this[1][1]);
        let v2 = util::float32x4x2_to_u16x8::<E>(this[0][2], this[1][2]);
        let v3 = util::float32x4x2_to_u16x8::<E>(this[0][3], this[1][3]);

        let v = transmute::<[[u16; 8]; 4], uint16x8x4_t>([v0, v1, v2, v3]);

        vst4q_u16(ptr, v)
    }

    #[inline(always)]
    unsafe fn dyn_rgba_read<'a>(
        v: &mut (dyn DynRgbaReader + 'a),
        x: usize,
        y: usize,
    ) -> RgbaBlock<Self> {
        DynRgbaReaderSpec::<float32x4_t>::dyn_read(v, x, y)
    }
}

mod math {
    use crate::arch::*;
    use crate::vector::Vector;
    use std::f32::consts::LOG2_E;
    use std::mem::transmute;

    const ONE: float32x4_t = splat(1.0);
    const ONE_HALF: float32x4_t = splat(0.5);

    const fn splat(f: f32) -> float32x4_t {
        unsafe { transmute([f; 4]) }
    }

    // Function copied from http://gruntthepeon.free.fr/ssemath/neon_mathfun.h
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn exp(x: float32x4_t) -> float32x4_t {
        const EXP_HI: float32x4_t = splat(88.376_26);
        const EXP_LO: float32x4_t = splat(-88.376_26);
        const CEPHES_LOG2EF: float32x4_t = splat(LOG2_E);
        const C_CEPHES_EXP_C1: float32x4_t = splat(0.693_359_4);
        const C_CEPHES_EXP_C2: float32x4_t = splat(-2.121_944_4e-4);

        const C_CEPHES_EXP_P0: float32x4_t = splat(1.987_569_1E-4);
        const C_CEPHES_EXP_P1: float32x4_t = splat(1.398_199_9E-3);
        const C_CEPHES_EXP_P2: float32x4_t = splat(8.333_452E-3);
        const C_CEPHES_EXP_P3: float32x4_t = splat(4.166_579_6E-2);
        const C_CEPHES_EXP_P4: float32x4_t = splat(1.666_666_6E-1);
        const C_CEPHES_EXP_P5: float32x4_t = splat(5E-1);

        let x = vminq_f32(x, EXP_HI);
        let x = vmaxq_f32(x, EXP_LO);

        /* express exp(x) as exp(g + n*log(2)) */
        let fx = vmlaq_f32(ONE_HALF, x, CEPHES_LOG2EF);

        /* perform a floorf */
        let tmp = vcvtq_f32_s32(vcvtq_s32_f32(fx));

        /* if greater, substract 1 */
        let mask = vcgtq_f32(tmp, fx);
        let mask = vandq_u32(mask, vreinterpretq_u32_f32(ONE));

        let fx = vsubq_f32(tmp, vreinterpretq_f32_u32(mask));

        let tmp = vmulq_f32(fx, C_CEPHES_EXP_C1);
        let z = vmulq_f32(fx, C_CEPHES_EXP_C2);
        let x = vsubq_f32(x, tmp);
        let x = vsubq_f32(x, z);

        let y = C_CEPHES_EXP_P0;
        let y = vmulq_f32(y, x);
        let z = vmulq_f32(x, x);
        let y = vaddq_f32(y, C_CEPHES_EXP_P1);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, C_CEPHES_EXP_P2);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, C_CEPHES_EXP_P3);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, C_CEPHES_EXP_P4);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, C_CEPHES_EXP_P5);

        let y = vmulq_f32(y, z);
        let y = vaddq_f32(y, x);
        let y = vaddq_f32(y, ONE);

        /* build 2^n */
        let mm = vcvtq_s32_f32(fx);
        let mm = vaddq_s32(mm, vdupq_n_s32(0x7f));
        let mm = vshlq_n_s32(mm, 23);
        let pow2n = vreinterpretq_f32_s32(mm);

        vmulq_f32(y, pow2n)
    }

    // Function copied from http://gruntthepeon.free.fr/ssemath/neon_mathfun.h
    #[target_feature(enable = "neon")]
    pub(super) unsafe fn log(x: float32x4_t) -> float32x4_t {
        const INV_MANT_MASK: i32 = !0x7f800000;
        const CEPHES_SQRT_HF: f32 = 0.707_106_77;
        const CEPHES_LOG_P0: float32x4_t = splat(7.037_683_6E-2);
        const CEPHES_LOG_P1: float32x4_t = splat(-1.151_461E-1);
        const CEPHES_LOG_P2: float32x4_t = splat(1.167_699_84E-1);
        const CEPHES_LOG_P3: float32x4_t = splat(-1.242_014_1E-1);
        const CEPHES_LOG_P4: float32x4_t = splat(1.424_932_3E-1);
        const CEPHES_LOG_P5: float32x4_t = splat(-1.666_805_7E-1);
        const CEPHES_LOG_P6: float32x4_t = splat(2.000_071_4E-1);
        const CEPHES_LOG_P7: float32x4_t = splat(-2.499_999_4E-1);
        const CEPHES_LOG_P8: float32x4_t = splat(3.333_333E-1);
        const CEPHES_LOG_Q1: float32x4_t = splat(-2.121_944_4e-4);
        const CEPHES_LOG_Q2: float32x4_t = splat(0.693_359_4);

        // TODO: Investigate what behavior for negative numbers is wanted here
        // if x is NaN then x == x = false
        let invalid_mask = vceqq_f32(x, x);

        let ux = vreinterpretq_s32_f32(x);

        let emm0 = vshrq_n_s32(ux, 23);

        /* keep only the fractional part */

        let ux = vandq_s32(ux, vdupq_n_s32(INV_MANT_MASK));
        let ux = vorrq_s32(ux, vreinterpretq_s32_f32(vdupq_n_f32(0.5)));
        let x = vreinterpretq_f32_s32(ux);

        let emm0 = vsubq_s32(emm0, vdupq_n_s32(0x7f));
        let e = vcvtq_f32_s32(emm0);

        let e = vaddq_f32(e, ONE);

        /* part2:
           if( x < SQRTHF ) {
             e -= 1;
             x = x + x - 1.0;
           } else { x = x - 1.0; }
        */
        let mask = vcltq_f32(x, vdupq_n_f32(CEPHES_SQRT_HF));
        let tmp = vreinterpretq_f32_u32(vandq_u32(vreinterpretq_u32_f32(x), mask));
        let x = vsubq_f32(x, ONE);
        let e = vsubq_f32(
            e,
            vreinterpretq_f32_u32(vandq_u32(vreinterpretq_u32_f32(ONE), mask)),
        );
        let x = vaddq_f32(x, tmp);

        let z = vmulq_f32(x, x);

        let y = vmulq_f32(CEPHES_LOG_P0, x);
        let y = vaddq_f32(y, CEPHES_LOG_P1);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P2);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P3);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P4);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P5);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P6);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P7);
        let y = vmulq_f32(y, x);
        let y = vaddq_f32(y, CEPHES_LOG_P8);
        let y = vmulq_f32(y, x);

        let y = vmulq_f32(y, z);

        let tmp = vmulq_f32(e, CEPHES_LOG_Q1);
        let y = vaddq_f32(y, tmp);

        let tmp = vmulq_f32(z, vdupq_n_f32(0.5));
        let y = vsubq_f32(y, tmp);

        let tmp = vmulq_f32(e, CEPHES_LOG_Q2);
        let x = vaddq_f32(x, y);
        let x = vaddq_f32(x, tmp);

        // Set invalid input to NaN
        float32x4_t::select(x, float32x4_t::splat(f32::NAN), invalid_mask)
    }

    #[inline(always)]
    pub(super) unsafe fn pow(x: float32x4_t, y: float32x4_t) -> float32x4_t {
        exp(vmulq_f32(y, log(x)))
    }
}

pub(crate) mod util {
    use crate::arch::*;
    use crate::endian::Endian;
    use std::mem::transmute;

    #[inline(always)]
    pub(crate) unsafe fn float32x4x2_to_u8x8(l: float32x4_t, h: float32x4_t) -> [u8; 8] {
        let l = vcvtq_u32_f32(l);
        let l = vminq_u32(l, vdupq_n_u32(u8::MAX as u32));
        let l = vmovn_u32(l);

        let h = vcvtq_u32_f32(h);
        let h = vminq_u32(h, vdupq_n_u32(u8::MAX as u32));
        let h = vmovn_u32(h);

        let v = transmute::<[uint16x4_t; 2], uint16x8_t>([l, h]);

        transmute(vmovn_u16(v))
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x4x2_to_u16x8<E: Endian>(
        l: float32x4_t,
        h: float32x4_t,
    ) -> [u16; 8] {
        let l = vcvtq_u32_f32(l);
        let l = vminq_u32(l, vdupq_n_u32(u16::MAX as u32));
        let l = vmovn_u32(l);

        let h = vcvtq_u32_f32(h);
        let h = vminq_u32(h, vdupq_n_u32(u16::MAX as u32));
        let h = vmovn_u32(h);

        let (l, h) = if E::IS_NATIVE {
            (l, h)
        } else {
            (vrev32_u16(l), (vrev32_u16(h)))
        };

        transmute([l, h])
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x4_to_u8x4(i: float32x4_t) -> [u8; 4] {
        let i = vcvtq_u32_f32(i);
        let i = vminq_u32(i, vdupq_n_u32(255));
        let i = vmovn_u32(i);

        let v = transmute::<[uint16x4_t; 2], uint16x8_t>([i, i]);

        let [a, b, c, d, ..] = transmute::<uint8x8_t, [u8; 8]>(vmovn_u16(v));

        [a, b, c, d]
    }

    #[inline(always)]
    pub(crate) unsafe fn float32x4_to_u16x4<E: Endian>(i: float32x4_t) -> [u16; 4] {
        let i = vcvtq_u32_f32(i);
        let i = vminq_u32(i, vdupq_n_u32(u16::MAX as u32));
        let i = vmovn_u32(i);

        if E::IS_NATIVE {
            transmute(i)
        } else {
            transmute(vrev32_u16(i))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            #[rustfmt::skip]
            let rgba = [
                100, 101, 102, 103,
                200, 201, 202, 203,
            ];

            let expected_a = [100.0, 200.0, 101.0, 201.0];
            let expected_b = [102.0, 202.0, 103.0, 203.0];

            let a = float32x4_t::load_u8(rgba.as_ptr());
            let b = float32x4_t::load_u8(rgba.as_ptr().add(4));

            let (a, b) = a.zip(b);

            assert_eq!(transmute::<_, [f32; 4]>(a), expected_a);
            assert_eq!(transmute::<_, [f32; 4]>(b), expected_b);
        }
    }

    #[test]
    fn unzip_for_rgb_to_i420_2x2block_add() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            #[rustfmt::skip]
            let red = [
                100, 101, 102, 103,  104, 105, 106, 107,
                200, 201, 202, 203,  204, 205, 206, 207,
            ];

            let result = [
                100 + 101 + 200 + 201,
                102 + 103 + 202 + 203,
                104 + 105 + 204 + 205,
                106 + 107 + 206 + 207,
            ]
            .map(|x| x as f32);

            let r00 = float32x4_t::load_u8(red.as_ptr());
            let r01 = float32x4_t::load_u8(red.as_ptr().add(float32x4_t::LEN));
            let r10 = float32x4_t::load_u8(red.as_ptr().add(float32x4_t::LEN * 2));
            let r11 = float32x4_t::load_u8(red.as_ptr().add(float32x4_t::LEN * 3));

            let r0 = r00.vadd(r10);
            let r1 = r01.vadd(r11);

            let (r0, r1) = r0.unzip(r1);

            let x = r0.vadd(r1);

            assert_eq!(transmute::<_, [f32; 4]>(x), result);
        }
    }

    #[track_caller]
    fn assert_within_error(a: f32, b: f32, error: f32) {
        assert!((a - b).abs() < error, "error too large between {a} and {b}")
    }

    unsafe fn make_arr(i: float32x4_t) -> [f32; 4] {
        transmute(i)
    }

    #[test]
    fn exp() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            assert!(make_arr(math::exp(float32x4_t::splat(f32::NAN)))[0].is_nan());

            for i in 0..10_000 {
                let i = (i - 5000) as f32 / 1000.0;

                let iv = float32x4_t::splat(i);

                let rv = math::exp(iv);
                let r = i.exp();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn log() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            assert!(make_arr(math::log(float32x4_t::splat(f32::NAN)))[0].is_nan());

            for i in 1..10_000 {
                let i = (i) as f32 / 10000.0;
                println!("{i}");

                let iv = float32x4_t::splat(i);

                let rv = math::log(iv);
                let r = i.ln();

                let rv = make_arr(rv)[0];

                assert_within_error(r, rv, 0.0001);
            }
        }
    }

    #[test]
    fn pow() {
        assert!(is_aarch64_feature_detected!("neon"));

        unsafe {
            assert!(make_arr(math::pow(
                float32x4_t::splat(f32::NAN),
                float32x4_t::splat(f32::NAN)
            ))[0]
                .is_nan());

            assert!(make_arr(math::pow(
                float32x4_t::splat(f32::NAN),
                float32x4_t::splat(1.0)
            ))[0]
                .is_nan());
            assert!(make_arr(math::pow(
                float32x4_t::splat(1.0),
                float32x4_t::splat(f32::NAN)
            ))[0]
                .is_nan());

            for a in 1..100 {
                for p in 1..1000 {
                    let a = (a) as f32 / 53.1;
                    let p = (p) as f32 / 839.333;
                    println!("{a}^{p}");

                    let rv = math::pow(float32x4_t::splat(a), float32x4_t::splat(p));
                    let r = a.powf(p);

                    let rv = make_arr(rv)[0];

                    assert_within_error(r, rv, 0.0001);
                }
            }
        }
    }
}
