use crate::endian::{BigEndian, Endian, LittleEndian, NativeEndian};
use crate::vector::Vector;

pub(crate) trait Bits {
    type Primitive;
    type Endian: Endian;

    fn primitive_from_f32(v: f32) -> Self::Primitive;

    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V;
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const Self::Primitive) -> [[V; 4]; 2];
}

pub(crate) struct U8;

impl Bits for U8 {
    type Primitive = u8;
    // Value doesn't matter
    type Endian = NativeEndian;

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

pub(crate) struct U16LE;

impl Bits for U16LE {
    type Primitive = u16;
    type Endian = LittleEndian;

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

pub(crate) struct U16BE;

impl Bits for U16BE {
    type Primitive = u16;
    type Endian = BigEndian;

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
