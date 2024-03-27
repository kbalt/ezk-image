use super::i420::{I420Block, I420VisitorImpl};
use crate::arch::*;
use crate::bits::{Bits, B8};
use crate::vector::Vector;
use crate::RawMutSliceU8;
use crate::Rect;
use std::marker::PhantomData;
use std::mem::size_of;

pub struct I420Writer<'a, B: Bits> {
    window: Rect,

    dst_width: usize,
    dst_height: usize,
    dst: *mut u8,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, B: Bits> I420Writer<'a, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst: RawMutSliceU8<'a>,
        window: Option<Rect>,
    ) -> Self {
        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: dst_width,
            height: dst_height,
        });

        assert!((dst_width * dst_height * 8 * size_of::<B::Primitive>()).div_ceil(12) <= dst.len());
        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            dst_height,
            dst: dst.ptr(),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<'a, B: Bits> I420VisitorImpl<f32> for I420Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<f32>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(B::MAX_VALUE);
        let y01 = y01.vmulf(B::MAX_VALUE);
        let y10 = y10.vmulf(B::MAX_VALUE);
        let y11 = y11.vmulf(B::MAX_VALUE);
        let u = u.vmulf(B::MAX_VALUE);
        let v = v.vmulf(B::MAX_VALUE);

        let y00 = B::primitive_from_f32(y00);
        let y01 = B::primitive_from_f32(y01);
        let y10 = B::primitive_from_f32(y10);
        let y11 = B::primitive_from_f32(y11);

        let offset00 = y * self.dst_width + x;
        let offset01 = y * self.dst_width + x + 1;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = (y + 1) * self.dst_width + x + 1;

        let dst = self.dst.cast::<B::Primitive>();

        dst.add(offset00).write(y00);
        dst.add(offset01).write(y01);
        dst.add(offset10).write(y10);
        dst.add(offset11).write(y11);

        let u = B::primitive_from_f32(u);
        let v = B::primitive_from_f32(v);

        let u_plane_offset = self.dst_width * self.dst_width;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        dst.add(u_plane_offset + uv_offset).write(u);
        dst.add(v_plane_offset + uv_offset).write(v);
    }
}

#[cfg(target_arch = "aarch64")]
impl<'a> I420VisitorImpl<float32x4_t> for I420Writer<'a, B8> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<float32x4_t>) {
        use crate::vector::neon::util::{float32x4_to_u8x4, float32x4x2_to_uint8x8_t};

        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(B8::MAX_VALUE);
        let y01 = y01.vmulf(B8::MAX_VALUE);
        let y10 = y10.vmulf(B8::MAX_VALUE);
        let y11 = y11.vmulf(B8::MAX_VALUE);
        let u = u.vmulf(B8::MAX_VALUE);
        let v = v.vmulf(B8::MAX_VALUE);

        let y0 = float32x4x2_to_uint8x8_t(y00, y01);
        let y1 = float32x4x2_to_uint8x8_t(y10, y11);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        self.dst.add(offset0).cast::<uint8x8_t>().write(y0);
        self.dst.add(offset1).cast::<uint8x8_t>().write(y1);

        let u = float32x4_to_u8x4(u);
        let v = float32x4_to_u8x4(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        self.dst
            .add(u_plane_offset + uv_offset)
            .cast::<[u8; 4]>()
            .write_unaligned(u);
        self.dst
            .add(v_plane_offset + uv_offset)
            .cast::<[u8; 4]>()
            .write_unaligned(v);
    }
}

#[cfg(target_arch = "aarch64")]
impl<'a, B: Bits<Primitive = u16>> I420VisitorImpl<float32x4_t> for I420Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<float32x4_t>) {
        use crate::vector::neon::util::{float32x4_to_u16x4, float32x4x2_to_uint16x8_t};

        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(B::MAX_VALUE);
        let y01 = y01.vmulf(B::MAX_VALUE);
        let y10 = y10.vmulf(B::MAX_VALUE);
        let y11 = y11.vmulf(B::MAX_VALUE);
        let u = u.vmulf(B::MAX_VALUE);
        let v = v.vmulf(B::MAX_VALUE);

        let y0 = float32x4x2_to_uint16x8_t::<B>(y00, y01);
        let y1 = float32x4x2_to_uint16x8_t::<B>(y10, y11);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        let dst = self.dst.cast::<B::Primitive>();

        dst.add(offset0).cast::<uint16x8_t>().write(y0);
        dst.add(offset1).cast::<uint16x8_t>().write(y1);

        let u = float32x4_to_u16x4::<B>(u);
        let v = float32x4_to_u16x4::<B>(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        dst.add(u_plane_offset + uv_offset)
            .cast::<uint16x4_t>()
            .write_unaligned(u);
        dst.add(v_plane_offset + uv_offset)
            .cast::<uint16x4_t>()
            .write_unaligned(v);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<'a> I420VisitorImpl<__m256> for I420Writer<'a, B8> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<__m256>) {
        use crate::vector::avx2::util::{float32x8_to_u8x8, float32x8x2_to_u8x16};

        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(255.0);
        let y01 = y01.vmulf(255.0);
        let y10 = y10.vmulf(255.0);
        let y11 = y11.vmulf(255.0);
        let u = u.vmulf(255.0);
        let v = v.vmulf(255.0);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        let y0 = float32x8x2_to_u8x16(y00, y01);
        let y1 = float32x8x2_to_u8x16(y10, y11);

        self.dst.add(offset0).cast::<[u8; 16]>().write(y0);
        self.dst.add(offset1).cast::<[u8; 16]>().write(y1);

        let u = float32x8_to_u8x8(u);
        let v = float32x8_to_u8x8(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        self.dst
            .add(u_plane_offset + uv_offset)
            .cast::<[u8; 8]>()
            .write(u);
        self.dst
            .add(v_plane_offset + uv_offset)
            .cast::<[u8; 8]>()
            .write(v);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<'a, B: Bits<Primitive = u16>> I420VisitorImpl<__m256> for I420Writer<'a, B> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<__m256>) {
        use crate::vector::avx2::util::{float32x8_to_u16x8, float32x8x2_to_u16x16};

        let x = self.window.x + x;
        let y = self.window.y + y;

        let I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        } = block;

        let y00 = y00.vmulf(B::MAX_VALUE);
        let y01 = y01.vmulf(B::MAX_VALUE);
        let y10 = y10.vmulf(B::MAX_VALUE);
        let y11 = y11.vmulf(B::MAX_VALUE);
        let u = u.vmulf(B::MAX_VALUE);
        let v = v.vmulf(B::MAX_VALUE);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        let y0 = float32x8x2_to_u16x16::<B::Endian>(y00, y01);
        let y1 = float32x8x2_to_u16x16::<B::Endian>(y10, y11);

        let dst = self.dst.cast::<u16>();

        dst.add(offset0).cast::<[u16; 16]>().write(y0);
        dst.add(offset1).cast::<[u16; 16]>().write(y1);

        let u = float32x8_to_u16x8::<B::Endian>(u);
        let v = float32x8_to_u16x8::<B::Endian>(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        dst.add(u_plane_offset + uv_offset)
            .cast::<[u16; 8]>()
            .write(u);
        dst.add(v_plane_offset + uv_offset)
            .cast::<[u16; 8]>()
            .write(v);
    }
}
