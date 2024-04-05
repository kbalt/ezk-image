use crate::color::{ColorOps, ColorOpsPart};
use crate::endian::Endian;
use std::fmt::Debug;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) mod avx2;

#[cfg(target_arch = "aarch64")]
pub(crate) mod neon;

/// Abstraction over float SIMD vector and common operations
///
/// # Safety
///
/// This trait is unsafe, because assumptions using constant `LEN` are made. Also every function is unsafe as they
/// might use instructions that are not available on the current processor, so cpu-feature checks always need to be made
/// before calling these.
///
/// Functions that are _only_ unsafe because they're generic over `Vector` are always safe to call with the `f32` type.
pub(crate) unsafe trait Vector: Debug + Copy + 'static {
    /// How many floats (f32) can this vector hold
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
    unsafe fn load_u8(ptr: *const u8) -> Self;

    /// Load LEN * 2 packed bytes and unpack them to f32 by scattering them to [b0, b1, 0, 0, b2, b3, 0, 0, b4, b5, 0, 0, ...]
    /// and then converting to floats
    ///
    /// # Safety
    ///
    /// Pointer must be valid to read Self::LEN * 2 bytes
    unsafe fn load_u16<E: Endian>(ptr: *const u16) -> Self;

    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2];
    unsafe fn load_u16_3x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 3]; 2];

    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2];
    unsafe fn load_u16_4x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 4]; 2];

    /// Write
    unsafe fn write_u8(self, ptr: *mut u8);
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8);
    unsafe fn write_u16<E: Endian>(self, ptr: *mut u16);
    unsafe fn write_u16_2x<E: Endian>(v0: Self, v1: Self, ptr: *mut u16);

    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8);
    unsafe fn write_interleaved_3x_2x_u16<E: Endian>(this: [[Self; 3]; 2], ptr: *mut u16);

    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8);
    unsafe fn write_interleaved_4x_2x_u16<E: Endian>(this: [[Self; 4]; 2], ptr: *mut u16);

    fn color_ops(c: &ColorOps) -> &ColorOpsPart<Self>;
}

unsafe impl Vector for f32 {
    const LEN: usize = 1;
    type Mask = bool;

    #[inline(always)]
    unsafe fn splat(v: f32) -> Self {
        v
    }

    #[inline(always)]
    unsafe fn vadd(self, other: Self) -> Self {
        self + other
    }
    #[inline(always)]
    unsafe fn vsub(self, other: Self) -> Self {
        self - other
    }
    #[inline(always)]
    unsafe fn vmul(self, other: Self) -> Self {
        self * other
    }
    #[inline(always)]
    unsafe fn vdiv(self, other: Self) -> Self {
        self / other
    }

    #[inline(always)]
    unsafe fn vmax(self, other: Self) -> Self {
        self.max(other)
    }

    #[inline(always)]
    unsafe fn lt(self, other: Self) -> Self::Mask {
        self < other
    }

    #[inline(always)]
    unsafe fn le(self, other: Self) -> Self::Mask {
        self <= other
    }
    #[inline(always)]
    unsafe fn select(a: Self, b: Self, mask: Self::Mask) -> Self {
        if mask {
            a
        } else {
            b
        }
    }

    #[inline(always)]
    unsafe fn vsqrt(self) -> Self {
        f32::sqrt(self)
    }

    #[inline(always)]
    unsafe fn vpow(self, pow: Self) -> Self {
        self.powf(pow)
    }
    #[inline(always)]
    unsafe fn vln(self) -> Self {
        self.ln()
    }

    #[inline(always)]
    unsafe fn zip(self, other: Self) -> (Self, Self) {
        (self, other)
    }

    #[inline(always)]
    unsafe fn unzip(self, other: Self) -> (Self, Self) {
        (self, other)
    }

    #[inline(always)]
    unsafe fn load_u8(ptr: *const u8) -> Self {
        Self::from(ptr.read_unaligned())
    }
    #[inline(always)]
    unsafe fn load_u16<E: Endian>(ptr: *const u16) -> Self {
        let v = ptr.read_unaligned();

        if E::IS_NATIVE {
            Self::from(v)
        } else {
            Self::from(v.swap_bytes())
        }
    }

    #[inline(always)]
    unsafe fn load_u8_3x_interleaved_2x(ptr: *const u8) -> [[Self; 3]; 2] {
        let v = ptr.cast::<[[u8; 3]; 2]>().read_unaligned();
        v.map(|v| v.map(|v| v as f32))
    }

    #[inline(always)]
    unsafe fn load_u16_3x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 3]; 2] {
        let v = ptr.cast::<[[u16; 3]; 2]>().read_unaligned();

        if E::IS_NATIVE {
            v.map(|v| v.map(|v| v as f32))
        } else {
            v.map(|v| v.map(|v| v.swap_bytes() as f32))
        }
    }

    #[inline(always)]
    unsafe fn load_u8_4x_interleaved_2x(ptr: *const u8) -> [[Self; 4]; 2] {
        let v = ptr.cast::<[[u8; 4]; 2]>().read_unaligned();
        v.map(|v| v.map(|v| v as f32))
    }

    #[inline(always)]
    unsafe fn load_u16_4x_interleaved_2x<E: Endian>(ptr: *const u16) -> [[Self; 4]; 2] {
        let v = ptr.cast::<[[u16; 4]; 2]>().read_unaligned();

        if E::IS_NATIVE {
            v.map(|v| v.map(|v| v as f32))
        } else {
            v.map(|v| v.map(|v| v.swap_bytes() as f32))
        }
    }

    #[inline(always)]
    unsafe fn write_u8(self, ptr: *mut u8) {
        ptr.write(self as u8)
    }
    #[inline(always)]
    unsafe fn write_u8_2x(v0: Self, v1: Self, ptr: *mut u8) {
        ptr.cast::<[u8; 2]>().write_unaligned([v0 as u8, v1 as u8]);
    }
    #[inline(always)]
    unsafe fn write_u16<E: Endian>(self, ptr: *mut u16) {
        let v = if E::IS_NATIVE {
            self as u16
        } else {
            (self as u16).swap_bytes()
        };
        ptr.write_unaligned(v);
    }
    #[inline(always)]
    unsafe fn write_u16_2x<E: Endian>(v0: Self, v1: Self, ptr: *mut u16) {
        let v = if E::IS_NATIVE {
            [v0, v1].map(|f| (f as u16))
        } else {
            [v0, v1].map(|f| (f as u16).swap_bytes())
        };

        ptr.cast::<[u16; 2]>().write_unaligned(v);
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u8(this: [[Self; 3]; 2], ptr: *mut u8) {
        ptr.cast::<[[u8; 3]; 2]>()
            .write_unaligned(this.map(|f| f.map(|f| f as u8)));
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x_u16<E: Endian>(this: [[Self; 3]; 2], ptr: *mut u16) {
        ptr.cast::<[[u16; 3]; 2]>().write_unaligned(this.map(|f| {
            f.map(|f| {
                if E::IS_NATIVE {
                    f as u16
                } else {
                    (f as u16).swap_bytes()
                }
            })
        }));
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u8(this: [[Self; 4]; 2], ptr: *mut u8) {
        ptr.cast::<[[u8; 4]; 2]>()
            .write_unaligned(this.map(|f| f.map(|f| f as u8)));
    }

    #[inline(always)]
    unsafe fn write_interleaved_4x_2x_u16<E: Endian>(this: [[Self; 4]; 2], ptr: *mut u16) {
        ptr.cast::<[[u16; 4]; 2]>().write_unaligned(this.map(|f| {
            f.map(|f| {
                if E::IS_NATIVE {
                    f as u16
                } else {
                    (f as u16).swap_bytes()
                }
            })
        }));
    }

    #[inline(always)]
    fn color_ops(c: &ColorOps) -> &ColorOpsPart<Self> {
        &c.f32
    }
}
