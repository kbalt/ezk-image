use crate::endian::{BigEndian, Endian, LittleEndian, NativeEndian};
use crate::vector::Vector;

pub trait Bits: 'static {
    type Primitive: Copy + Send + Sync + 'static;
    type Endian: Endian;
}

pub(crate) trait BitsInternal: Bits {
    type FirPixel1: fir::pixels::PixelExt;
    type FirPixel2: fir::pixels::PixelExt;
    type FirPixel3: fir::pixels::PixelExt;
    type FirPixel4: fir::pixels::PixelExt;

    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V;
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 3]; 2];
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2];

    unsafe fn write<V: Vector>(ptr: *mut Self::Primitive, v: V);
    unsafe fn write_2x<V: Vector>(ptr: *mut Self::Primitive, v0: V, v1: V);

    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 3]; 2]);
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 4]; 2]);

    fn swap_bytes(b: &mut [Self::Primitive]);
}

pub struct U8;

impl Bits for U8 {
    type Primitive = u8;
    // Value doesn't matter
    type Endian = NativeEndian;
}

impl BitsInternal for U8 {
    type FirPixel1 = fir::pixels::U8;
    type FirPixel2 = fir::pixels::U8x2;
    type FirPixel3 = fir::pixels::U8x3;
    type FirPixel4 = fir::pixels::U8x4;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u8(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut Self::Primitive, v: V) {
        v.write_u8(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut Self::Primitive, v0: V, v1: V) {
        V::write_u8_2x(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 3]; 2] {
        V::load_u8_3x_interleaved_2x(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u8_4x_interleaved_2x(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u8(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u8(v, ptr)
    }

    #[inline(always)]
    fn swap_bytes(_: &mut [Self::Primitive]) {}
}

pub struct U16LE;

impl Bits for U16LE {
    type Primitive = u16;
    type Endian = LittleEndian;
}

impl BitsInternal for U16LE {
    type FirPixel1 = fir::pixels::U16;
    type FirPixel2 = fir::pixels::U16x2;
    type FirPixel3 = fir::pixels::U16x3;
    type FirPixel4 = fir::pixels::U16x4;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<LittleEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut Self::Primitive, v: V) {
        v.write_u16::<Self::Endian>(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut Self::Primitive, v0: V, v1: V) {
        V::write_u16_2x::<Self::Endian>(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 3]; 2] {
        V::load_u16_3x_interleaved_2x::<Self::Endian>(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<LittleEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u16::<Self::Endian>(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u16::<Self::Endian>(v, ptr)
    }

    fn swap_bytes(b: &mut [Self::Primitive]) {
        swap_bytes(b)
    }
}

pub struct U16BE;

impl Bits for U16BE {
    type Primitive = u16;
    type Endian = BigEndian;
}

impl BitsInternal for U16BE {
    // TODO: probably wrong :/
    type FirPixel1 = fir::pixels::U16;
    type FirPixel2 = fir::pixels::U16x2;
    type FirPixel3 = fir::pixels::U16x3;
    type FirPixel4 = fir::pixels::U16x4;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<BigEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(ptr: *mut Self::Primitive, v: V) {
        v.write_u16::<Self::Endian>(ptr)
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(ptr: *mut Self::Primitive, v0: V, v1: V) {
        V::write_u16_2x::<Self::Endian>(v0, v1, ptr)
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 3]; 2] {
        V::load_u16_3x_interleaved_2x::<Self::Endian>(ptr)
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<BigEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 3]; 2]) {
        V::write_interleaved_3x_2x_u16::<Self::Endian>(v, ptr)
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut Self::Primitive, v: [[V; 4]; 2]) {
        V::write_interleaved_4x_2x_u16::<Self::Endian>(v, ptr)
    }

    fn swap_bytes(b: &mut [Self::Primitive]) {
        swap_bytes(b)
    }
}

fn swap_bytes(b: &mut [u16]) {
    #[inline(always)]
    fn impl_(b: &mut [u16]) {
        for b in b {
            *b = b.swap_bytes();
        }
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
    if is_x86_feature_detected!("neon") {
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
