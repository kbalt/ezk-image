use crate::endian::{BigEndian, Endian, LittleEndian, NativeEndian};
use crate::vector::Vector;

pub(crate) trait Bits {
    type Primitive;
    type Endian: Endian;
    const MAX_VALUE: f32;

    fn primitive_from_f32(v: f32) -> Self::Primitive;

    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V;
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2];
}

pub(crate) struct B8;

impl Bits for B8 {
    type Primitive = u8;
    // Value doesn't matter
    type Endian = NativeEndian;
    const MAX_VALUE: f32 = 255.0;

    fn primitive_from_f32(v: f32) -> Self::Primitive {
        v as u8
    }

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u8(ptr)
    }

    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u8_4x_interleaved_2x(ptr)
    }
}

pub(crate) struct B10LittleEndian;

impl Bits for B10LittleEndian {
    type Primitive = u16;
    type Endian = LittleEndian;
    const MAX_VALUE: f32 = 1023.0;

    fn primitive_from_f32(v: f32) -> Self::Primitive {
        (v as u16).to_le()
    }

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<LittleEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<LittleEndian>(ptr)
    }
}

pub(crate) struct B12LittleEndian;

impl Bits for B12LittleEndian {
    type Primitive = u16;
    type Endian = LittleEndian;
    const MAX_VALUE: f32 = 4095.0;

    fn primitive_from_f32(v: f32) -> Self::Primitive {
        (v as u16).to_le()
    }

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<LittleEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<LittleEndian>(ptr)
    }
}

pub(crate) struct B10BigEndian;

impl Bits for B10BigEndian {
    type Primitive = u16;
    type Endian = BigEndian;
    const MAX_VALUE: f32 = 1023.0;

    fn primitive_from_f32(v: f32) -> Self::Primitive {
        (v as u16).to_be()
    }

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<BigEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<BigEndian>(ptr)
    }
}

pub(crate) struct B12BigEndian;

impl Bits for B12BigEndian {
    type Primitive = u16;
    type Endian = BigEndian;
    const MAX_VALUE: f32 = 4095.0;

    fn primitive_from_f32(v: f32) -> Self::Primitive {
        (v as u16).to_be()
    }

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<BigEndian>(ptr)
    }

    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2] {
        V::load_u16_4x_interleaved_2x::<BigEndian>(ptr)
    }
}
