use crate::vector::Vector;

pub(crate) trait Primitive {
    const SIZE: usize;

    unsafe fn load<V: Vector>(slice: &[u8]) -> V;
    unsafe fn load_3x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 3]; 2];
    unsafe fn load_4x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 4]; 2];

    unsafe fn write<V: Vector>(slice: &mut [u8], v: V);
    unsafe fn write_2x<V: Vector>(slice: &mut [u8], v0: V, v1: V);

    unsafe fn write_interleaved_3x_2x<V: Vector>(slice: &mut [u8], v: [[V; 3]; 2]);
    unsafe fn write_interleaved_4x_2x<V: Vector>(slice: &mut [u8], v: [[V; 4]; 2]);
}

impl Primitive for u8 {
    const SIZE: usize = 1;

    #[inline(always)]
    unsafe fn load<V: Vector>(slice: &[u8]) -> V {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE);
        V::load_u8(slice.as_ptr())
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 3]; 2] {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 3 * 2);
        V::load_u8_3x_interleaved_2x(slice.as_ptr())
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 4]; 2] {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 4 * 2);
        V::load_u8_4x_interleaved_2x(slice.as_ptr())
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(slice: &mut [u8], v: V) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE);
        v.write_u8(slice.as_mut_ptr())
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(slice: &mut [u8], v0: V, v1: V) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 2);
        V::write_u8_2x(v0, v1, slice.as_mut_ptr())
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(slice: &mut [u8], v: [[V; 3]; 2]) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 3 * 2);
        V::write_interleaved_3x_2x_u8(v, slice.as_mut_ptr())
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(slice: &mut [u8], v: [[V; 4]; 2]) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 4 * 2);
        V::write_interleaved_4x_2x_u8(v, slice.as_mut_ptr())
    }
}

impl Primitive for u16 {
    const SIZE: usize = 2;

    #[inline(always)]
    unsafe fn load<V: Vector>(slice: &[u8]) -> V {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE);
        V::load_u16(slice.as_ptr())
    }

    #[inline(always)]
    unsafe fn load_3x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 3]; 2] {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 3 * 2);
        V::load_u16_3x_interleaved_2x(slice.as_ptr())
    }
    #[inline(always)]
    unsafe fn load_4x_interleaved_2x<V: Vector>(slice: &[u8]) -> [[V; 4]; 2] {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 4 * 2);
        V::load_u16_4x_interleaved_2x(slice.as_ptr())
    }

    #[inline(always)]
    unsafe fn write<V: Vector>(slice: &mut [u8], v: V) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE);
        v.write_u16(slice.as_mut_ptr())
    }
    #[inline(always)]
    unsafe fn write_2x<V: Vector>(slice: &mut [u8], v0: V, v1: V) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 2);
        V::write_u16_2x(v0, v1, slice.as_mut_ptr())
    }

    #[inline(always)]
    unsafe fn write_interleaved_3x_2x<V: Vector>(slice: &mut [u8], v: [[V; 3]; 2]) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 3 * 2);
        V::write_interleaved_3x_2x_u16(v, slice.as_mut_ptr())
    }
    #[inline(always)]
    unsafe fn write_interleaved_4x_2x<V: Vector>(slice: &mut [u8], v: [[V; 4]; 2]) {
        debug_assert!(slice.len() >= V::LEN * Self::SIZE * 4 * 2);
        V::write_interleaved_4x_2x_u16(v, slice.as_mut_ptr())
    }
}
