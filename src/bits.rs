use crate::endian::{BigEndian, Endian, LittleEndian};
use crate::vector::Vector;

pub(crate) trait Bits {
    type Primitive;
    const MAX_VALUE: f32;

    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V;
}

pub(crate) struct B8;

impl Bits for B8 {
    type Primitive = u8;
    const MAX_VALUE: f32 = 255.0;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u8(ptr)
    }
}

pub(crate) struct B10LittleEndian;

impl Bits for B10LittleEndian {
    type Primitive = u16;
    const MAX_VALUE: f32 = 1023.0;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<LittleEndian>(ptr)
    }
}

pub(crate) struct B12LittleEndian;

impl Bits for B12LittleEndian {
    type Primitive = u16;
    const MAX_VALUE: f32 = 4095.0;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<LittleEndian>(ptr)
    }
}

pub(crate) struct B10BigEndian;

impl Bits for B10BigEndian {
    type Primitive = u16;
    const MAX_VALUE: f32 = 1023.0;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<BigEndian>(ptr)
    }
}

pub(crate) struct B12BigEndian;

impl Bits for B12BigEndian {
    type Primitive = u16;
    const MAX_VALUE: f32 = 4095.0;

    #[inline(always)]
    unsafe fn load<V: Vector>(ptr: *const Self::Primitive) -> V {
        V::load_u16::<BigEndian>(ptr)
    }
}
