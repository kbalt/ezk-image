use crate::vector::Vector;

pub(crate) trait PrimitiveInternal {
    const SIZE: usize;

    unsafe fn load<V: Vector>(ptr: *const u8) -> V;
    unsafe fn load_3x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 3]; 2];
    unsafe fn load_4x_interleaved_2x<V: Vector>(ptr: *const u8) -> [[V; 4]; 2];

    unsafe fn write<V: Vector>(ptr: *mut u8, v: V);
    unsafe fn write_2x<V: Vector>(ptr: *mut u8, v0: V, v1: V);

    unsafe fn write_interleaved_3x_2x<V: Vector>(ptr: *mut u8, v: [[V; 3]; 2]);
    unsafe fn write_interleaved_4x_2x<V: Vector>(ptr: *mut u8, v: [[V; 4]; 2]);
}

impl PrimitiveInternal for u8 {
    const SIZE: usize = 1;

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

impl PrimitiveInternal for u16 {
    const SIZE: usize = 2;

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
