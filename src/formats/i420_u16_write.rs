use super::i420::{I420Block, I420VisitorImpl};
use crate::arch::*;
use crate::endian::Endian;
use crate::util::scale;
use crate::vector::Vector;
use crate::{RawMutSliceU8, Rect};
use std::marker::PhantomData;

pub struct I420U16Writer<'a, const BIT_DEPTH: usize, E: Endian> {
    window: Rect,

    dst_width: usize,
    dst_height: usize,
    dst: *mut u16,

    _m: PhantomData<&'a mut [u8]>,
    _e: PhantomData<E>,
}

impl<'a, const BIT_DEPTH: usize, E: Endian> I420U16Writer<'a, BIT_DEPTH, E> {
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

        // TODO: xd assert!(PixelFormat::I420P10BE.buffer_size(dst_width, dst_height) <= dst.len());
        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            dst_height,
            dst: dst.ptr().cast(),
            _m: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<'a, const BIT_DEPTH: usize, E: Endian> I420VisitorImpl<f32>
    for I420U16Writer<'a, BIT_DEPTH, E>
{
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

        let scale = scale(BIT_DEPTH);

        let y00 = y00.vmulf(scale);
        let y01 = y01.vmulf(scale);
        let y10 = y10.vmulf(scale);
        let y11 = y11.vmulf(scale);
        let u = u.vmulf(scale);
        let v = v.vmulf(scale);

        let y00 = y00 as u16;
        let y01 = y01 as u16;
        let y10 = y10 as u16;
        let y11 = y11 as u16;
        let u = u as u16;
        let v = v as u16;

        let (y00, y01, y10, y11, u, v) = if E::IS_NATIVE {
            (y00, y01, y10, y11, u, v)
        } else {
            (
                y00.swap_bytes(),
                y01.swap_bytes(),
                y10.swap_bytes(),
                y11.swap_bytes(),
                u.swap_bytes(),
                v.swap_bytes(),
            )
        };

        let offset00 = y * self.dst_width + x;
        let offset01 = y * self.dst_width + x + 1;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = (y + 1) * self.dst_width + x + 1;

        self.dst.add(offset00).write_unaligned(y00);
        self.dst.add(offset01).write_unaligned(y01);
        self.dst.add(offset10).write_unaligned(y10);
        self.dst.add(offset11).write_unaligned(y11);

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
impl<'a, const BIT_DEPTH: usize, E: Endian> I420VisitorImpl<float32x4_t>
    for I420U16Writer<'a, BIT_DEPTH, E>
{
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

        let scale = scale(BIT_DEPTH);

        let y00 = y00.vmulf(scale);
        let y01 = y01.vmulf(scale);
        let y10 = y10.vmulf(scale);
        let y11 = y11.vmulf(scale);
        let u = u.vmulf(scale);
        let v = v.vmulf(scale);

        let y0 = float32x4x2_to_uint16x8_t::<BIT_DEPTH, E>(y00, y01);
        let y1 = float32x4x2_to_uint16x8_t::<BIT_DEPTH, E>(y10, y11);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        self.dst.add(offset0).cast::<uint16x8_t>().write(y0);
        self.dst.add(offset1).cast::<uint16x8_t>().write(y1);

        let u = float32x4_to_u16x4::<BIT_DEPTH, E>(u);
        let v = float32x4_to_u16x4::<BIT_DEPTH, E>(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        self.dst
            .add(u_plane_offset + uv_offset)
            .cast::<uint16x4_t>()
            .write_unaligned(u);
        self.dst
            .add(v_plane_offset + uv_offset)
            .cast::<uint16x4_t>()
            .write_unaligned(v);
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<'a, const BIT_DEPTH: usize, E: Endian> I420VisitorImpl<__m256>
    for I420U16Writer<'a, BIT_DEPTH, E>
{
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

        let scale = scale(BIT_DEPTH);

        let y00 = y00.vmulf(scale);
        let y01 = y01.vmulf(scale);
        let y10 = y10.vmulf(scale);
        let y11 = y11.vmulf(scale);
        let u = u.vmulf(scale);
        let v = v.vmulf(scale);

        let offset0 = y * self.dst_width + x;
        let offset1 = (y + 1) * self.dst_width + x;

        let y0 = float32x8x2_to_u16x16::<E>(y00, y01);
        let y1 = float32x8x2_to_u16x16::<E>(y10, y11);

        self.dst.add(offset0).cast::<[u16; 16]>().write(y0);
        self.dst.add(offset1).cast::<[u16; 16]>().write(y1);

        let u = float32x8_to_u16x8::<E>(u);
        let v = float32x8_to_u16x8::<E>(v);

        let u_plane_offset = self.dst_width * self.dst_height;
        let v_plane_offset = u_plane_offset + (u_plane_offset / 4);

        let hx = x / 2;
        let hy = y / 2;
        let hw = self.dst_width / 2;

        let uv_offset = (hy * hw) + hx;
        self.dst
            .add(u_plane_offset + uv_offset)
            .cast::<[u16; 8]>()
            .write(u);
        self.dst
            .add(v_plane_offset + uv_offset)
            .cast::<[u16; 8]>()
            .write(v);
    }
}
