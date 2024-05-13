use crate::vector::Vector;

#[allow(private_bounds)]
pub trait Primitive: PrimitiveInternal + Copy + Send + Sync + 'static {}

pub(crate) trait PrimitiveInternal {
    #[cfg(feature = "resize")]
    type FirPixel1: fir::pixels::PixelExt;
    #[cfg(feature = "resize")]
    type FirPixel2: fir::pixels::PixelExt;
    #[cfg(feature = "resize")]
    type FirPixel3: fir::pixels::PixelExt;
    #[cfg(feature = "resize")]
    type FirPixel4: fir::pixels::PixelExt;

    unsafe fn load<V: Vector>(ptr: *const Self) -> V;
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 3]; 2];
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 4]; 2];

    unsafe fn write<V: Vector>(ptr: *mut Self, v: V);
    unsafe fn write_2x<V: Vector>(ptr: *mut Self, v0: V, v1: V);

    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self, v: [[V; 3]; 2]);
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self, v: [[V; 4]; 2]);
}

impl Primitive for u8 {}

impl PrimitiveInternal for u8 {
    #[cfg(feature = "resize")]
    type FirPixel1 = fir::pixels::U8;
    #[cfg(feature = "resize")]
    type FirPixel2 = fir::pixels::U8x2;
    #[cfg(feature = "resize")]
    type FirPixel3 = fir::pixels::U8x3;
    #[cfg(feature = "resize")]
    type FirPixel4 = fir::pixels::U8x4;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self) -> V {
        V::load_u8(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut Self, v: V) {
        v.write_u8(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut Self, v0: V, v1: V) {
        V::write_u8_2x(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 3]; 2] {
        V::load_u8_3x_interleaved_2x(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 4]; 2] {
        V::load_u8_4x_interleaved_2x(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u8(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u8(v, ptr)
    }
}

impl Primitive for u16 {}

impl PrimitiveInternal for u16 {
    #[cfg(feature = "resize")]
    type FirPixel1 = fir::pixels::U16;
    #[cfg(feature = "resize")]
    type FirPixel2 = fir::pixels::U16x2;
    #[cfg(feature = "resize")]
    type FirPixel3 = fir::pixels::U16x3;
    #[cfg(feature = "resize")]
    type FirPixel4 = fir::pixels::U16x4;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self) -> V {
        V::load_u16(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut Self, v: V) {
        v.write_u16(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut Self, v0: V, v1: V) {
        V::write_u16_2x(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 3]; 2] {
        V::load_u16_3x_interleaved_2x(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u16(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u16(v, ptr)
    }
}

pub(crate) fn swap_bytes(b: &mut [u16]) {
    #[inline(always)]
    fn impl_(b: &mut [u16]) {
        for b in b {
            *b = b.swap_bytes();
        }
    }

    #[cfg(all(feature = "unstable", any(target_arch = "x86", target_arch = "x86_64")))]
    if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
        #[target_feature(enable = "avx512f", enable = "avx512bw")]
        unsafe fn call(b: &mut [u16]) {
            impl_(b);
        }

        // Safety: Did a feature check
        unsafe { call(b) }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        #[target_feature(enable = "avx2")]
        unsafe fn call(b: &mut [u16]) {
            impl_(b);
        }

        // Safety: Did a feature check
        unsafe { call(b) }
    }

    #[cfg(target_arch = "aarch64")]
    if crate::arch::is_aarch64_feature_detected!("neon") {
        #[target_feature(enable = "neon")]
        unsafe fn call(b: &mut [u16]) {
            impl_(b);
        }

        // Safety: Did a feature check
        unsafe {
            call(b);
        }
    }

    impl_(b)
}
