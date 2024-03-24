use crate::color::{ColorOps, ColorOpsPart};
use std::fmt::Debug;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) mod avx2;

#[cfg(target_arch = "aarch64")]
pub(crate) mod neon;

/// Abstraction over float SIMD vector and common operations
pub(crate) unsafe trait Vector: Debug + Copy + 'static {
    /// Many pixels (f32) can this vector hold
    const LEN: usize;
    type Mask;

    /// Set all elements in the vector to the given value
    unsafe fn splat(v: f32) -> Self;

    unsafe fn vadd(self, other: Self) -> Self;
    unsafe fn vaddf(self, other: f32) -> Self {
        self.vadd(Self::splat(other))
    }

    unsafe fn vsub(self, other: Self) -> Self;
    unsafe fn vsubf(self, other: f32) -> Self {
        self.vsub(Self::splat(other))
    }

    unsafe fn vmul(self, other: Self) -> Self;
    unsafe fn vmulf(self, other: f32) -> Self {
        self.vmul(Self::splat(other))
    }

    unsafe fn vdiv(self, other: Self) -> Self;
    unsafe fn vdivf(self, other: f32) -> Self {
        self.vdiv(Self::splat(other))
    }

    unsafe fn vmax(self, other: Self) -> Self;
    unsafe fn vmaxf(self, other: f32) -> Self {
        self.vmax(Self::splat(other))
    }

    /// Compare all element in self to other and return a mask with the results a < b
    unsafe fn lt(self, other: Self) -> Self::Mask;
    unsafe fn ltf(self, other: f32) -> Self::Mask {
        self.lt(Self::splat(other))
    }

    /// Compare all element in self to other and return a mask with the results a <= b
    unsafe fn le(self, other: Self) -> Self::Mask;
    unsafe fn lef(self, other: f32) -> Self::Mask {
        self.le(Self::splat(other))
    }

    /// Select element in either a or b as specified in mask
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self;

    /// Calculate the square root of self
    unsafe fn vsqrt(self) -> Self;
    /// Calculate self ^ pow
    unsafe fn vpow(self, pow: Self) -> Self;
    unsafe fn vpowf(self, pow: f32) -> Self {
        self.vpow(Self::splat(pow))
    }

    /// Take the natural log of all f32 in the vector
    unsafe fn vln(self) -> Self;

    /// Interleave self and other
    unsafe fn zip(self, other: Self) -> (Self, Self);

    /// Given  [a0, a1, a2, a3] and [b0, b1, b2, b3]
    ///
    /// Return [a0, a2, b0, b2] and [a1, a3, b1, b3]
    unsafe fn unzip(self, other: Self) -> (Self, Self);

    /// Load LEN packed bytes and unpack them to f32 by scattering them to [b0, 0, 0, 0, b1, 0, 0, 0, b2, 0, 0, 0, ...]
    /// and then converting to floats
    ///
    /// # Safety
    ///
    /// Pointer must be valid to read Self::LEN bytes
    unsafe fn load(ptr: *const u8) -> Self;

    fn color_ops(c: &ColorOps) -> &ColorOpsPart<Self>;
}

unsafe impl Vector for f32 {
    const LEN: usize = 1;
    type Mask = bool;

    unsafe fn splat(v: f32) -> Self {
        v
    }

    unsafe fn vadd(self, other: Self) -> Self {
        self + other
    }
    unsafe fn vsub(self, other: Self) -> Self {
        self - other
    }
    unsafe fn vmul(self, other: Self) -> Self {
        self * other
    }
    unsafe fn vdiv(self, other: Self) -> Self {
        self / other
    }

    unsafe fn vmax(self, other: Self) -> Self {
        self.max(other)
    }

    unsafe fn lt(self, other: Self) -> Self::Mask {
        self < other
    }

    unsafe fn le(self, other: Self) -> Self::Mask {
        self <= other
    }
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self {
        if mask {
            a
        } else {
            b
        }
    }

    unsafe fn vsqrt(self) -> Self {
        f32::sqrt(self)
    }

    unsafe fn vpow(self, pow: Self) -> Self {
        if cfg!(feature = "veryfastmath") {
            veryfastmath::pow(self, pow)
        } else if cfg!(feature = "fastmath") {
            fastmath::pow(self, pow)
        } else {
            self.powf(pow)
        }
    }
    unsafe fn vln(self) -> Self {
        if cfg!(feature = "veryfastmath") {
            veryfastmath::log(self)
        } else if cfg!(feature = "fastmath") {
            fastmath::log(self)
        } else {
            self.ln()
        }
    }

    unsafe fn zip(self, other: Self) -> (Self, Self) {
        (self, other)
    }

    unsafe fn unzip(self, other: Self) -> (Self, Self) {
        (self, other)
    }

    unsafe fn load(ptr: *const u8) -> Self {
        Self::from(ptr.read_unaligned())
    }

    #[inline(always)]
    fn color_ops(c: &ColorOps) -> &ColorOpsPart<Self> {
        &c.f32
    }
}

mod fastmath {
    // Ported from https://code.google.com/archive/p/fastapprox/

    use std::f32::consts::{LN_2, LOG2_E};

    pub(super) fn exp(x: f32) -> f32 {
        let x = LOG2_E * x;

        let offset = if x < 0.0 { 1.0 } else { 0.0 };

        let clip = x.max(-126.0);
        let z = clip.fract() + offset;

        let i = (((1 << 23) as f32)
            * (clip + 121.274_055 + 27.728_024 / (4.842_525_5 - z) - 1.490_129_1 * z))
            as u32;

        f32::from_bits(i)
    }

    pub(super) fn log(x: f32) -> f32 {
        let vx_i = x.to_bits();

        let mx_i = (vx_i & 0x007FFFFF) | 0x3f000000;
        let y = (vx_i as f32) * 1.192_092_9e-7;

        let mx_f = f32::from_bits(mx_i);

        let r = y - 124.225_52 - 1.498_030_3 * mx_f - 1.725_88 / (0.352_088_72 + mx_f);

        r * LN_2
    }

    pub(super) fn pow(x: f32, y: f32) -> f32 {
        exp(x * log(y))
    }
}

mod veryfastmath {
    // Ported from https://code.google.com/archive/p/fastapprox/

    use std::f32::consts::{LN_2, LOG2_E};

    pub(super) fn exp(x: f32) -> f32 {
        let x = LOG2_E * x;

        let clip = x.max(-126.0);

        let i = (((1 << 23) as f32) * (clip + 126.942_696)) as u32;

        f32::from_bits(i)
    }

    pub(super) fn log(x: f32) -> f32 {
        let vx_i = x.to_bits();

        let mx_i = (vx_i & 0x007FFFFF) | 0x3f000000;
        let y = (vx_i as f32) * 1.192_092_9e-7;

        let mx_f = f32::from_bits(mx_i);

        let r = y - 124.225_52 - 1.498_030_3 * mx_f - 1.725_88 / (0.352_088_72 + mx_f);

        r * LN_2
    }

    pub(super) fn pow(x: f32, y: f32) -> f32 {
        exp(x * log(y))
    }
}
