use super::i420::{I420Block, I420VisitorImpl};
use crate::arch::*;
use crate::vector::Vector;
use crate::RawMutSliceU8;
use crate::Rect;
use std::marker::PhantomData;

pub struct I420U8Writer<'a> {
    window: Rect,

    dst_width: usize,
    dst_height: usize,
    dst: *mut u8,

    _m: PhantomData<&'a mut [u8]>,
}

impl<'a> I420U8Writer<'a> {
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

        assert!((dst_width * dst_height * 8).div_ceil(12) <= dst.len());
        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            dst_height,
            dst: dst.ptr(),
            _m: PhantomData,
        }
    }
}

impl<'a> I420VisitorImpl<f32> for I420U8Writer<'a> {
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

        let y00 = y00.vmulf(255.0);
        let y01 = y01.vmulf(255.0);
        let y10 = y10.vmulf(255.0);
        let y11 = y11.vmulf(255.0);
        let u = u.vmulf(255.0);
        let v = v.vmulf(255.0);

        let y00 = y00 as u8;
        let y01 = y01 as u8;
        let y10 = y10 as u8;
        let y11 = y11 as u8;

        let offset00 = y * self.dst_width + x;
        let offset01 = y * self.dst_width + x + 1;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = (y + 1) * self.dst_width + x + 1;

        self.dst.add(offset00).write(y00);
        self.dst.add(offset01).write(y01);
        self.dst.add(offset10).write(y10);
        self.dst.add(offset11).write(y11);

        let u = u as u8;
        let v = v as u8;

        let u_plane_offset = self.dst_width * self.dst_width;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        self.dst.add(u_plane_offset + uv_offset).write(u);
        self.dst.add(v_plane_offset + uv_offset).write(v);
    }
}

#[cfg(target_arch = "aarch64")]
impl<'a> I420VisitorImpl<float32x4_t> for I420U8Writer<'a> {
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

        let y00 = y00.vmulf(255.0);
        let y01 = y01.vmulf(255.0);
        let y10 = y10.vmulf(255.0);
        let y11 = y11.vmulf(255.0);
        let u = u.vmulf(255.0);
        let v = v.vmulf(255.0);

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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<'a> I420VisitorImpl<__m256> for I420U8Writer<'a> {
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
