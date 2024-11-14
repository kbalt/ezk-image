use crate::vector::Vector;

#[allow(private_bounds)]
// pub trait Primitive: PrimitiveInternal + Copy + Send + Sync + 'static {}

pub(crate) trait PrimitiveInternal {
    #[cfg(feature = "resize")]
    type FirPixel1: fir::PixelTrait;
    #[cfg(feature = "resize")]
    type FirPixel2: fir::PixelTrait;
    #[cfg(feature = "resize")]
    type FirPixel3: fir::PixelTrait;
    #[cfg(feature = "resize")]
    type FirPixel4: fir::PixelTrait;

    unsafe fn load<V: Vector>(ptr: *const u8) -> V;
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 3]; 2];
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 4]; 2];

    unsafe fn write<V: Vector>(ptr: *mut u8, v: V);
    unsafe fn write_2x<V: Vector>(ptr: *mut u8, v0: V, v1: V);

    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut u8, v: [[V; 3]; 2]);
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut u8, v: [[V; 4]; 2]);
}

// impl Primitive for u8 {}

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
    unsafe fn load<V: Vector>(ptr: *const u8) -> V {
        V::load_u8(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut u8, v: V) {
        v.write_u8(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut u8, v0: V, v1: V) {
        V::write_u8_2x(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 3]; 2] {
        V::load_u8_3x_interleaved_2x(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 4]; 2] {
        V::load_u8_4x_interleaved_2x(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut u8, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u8(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut u8, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u8(v, ptr)
    }
}

// impl Primitive for u16 {}

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
    unsafe fn load<V: Vector>(ptr: *const u8) -> V {
        V::load_u16(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut u8, v: V) {
        v.write_u16(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut u8, v0: V, v1: V) {
        V::write_u16_2x(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 3]; 2] {
        V::load_u16_3x_interleaved_2x(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut u8, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u16(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut u8, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u16(v, ptr)
    }
}
