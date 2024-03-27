use super::rgb::{RgbBlock, RgbBlockVisitorImpl, RgbPixel};
use super::rgba::{RgbaBlock, RgbaBlockVisitorImpl, RgbaPixel};
use crate::vector::Vector;
use crate::{arch::*, RawMutSliceU8, Rect};
use std::marker::PhantomData;

/// Writes 4 Bytes for every visited pixel in R G B (A/X) order
pub(crate) struct RGBAWriter<'a, const REVERSE: bool = false> {
    window: Rect,

    dst_width: usize,
    dst: *mut u8,

    _m: PhantomData<&'a mut [u8]>,
}

impl<'a, const REVERSE: bool> RGBAWriter<'a, REVERSE> {
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

        assert!(dst_width * dst_height * 4 <= dst.len());
        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            dst: dst.ptr(),
            _m: PhantomData,
        }
    }
}

impl<const REVERSE: bool, V: Vector> RgbBlockVisitorImpl<V> for RGBAWriter<'_, REVERSE>
where
    Self: RgbaBlockVisitorImpl<V>,
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        unsafe fn conv<V: Vector>(px: RgbPixel<V>) -> RgbaPixel<V> {
            RgbaPixel {
                r: px.r,
                g: px.g,
                b: px.b,
                a: V::splat(1.0),
            }
        }

        RgbaBlockVisitorImpl::visit(
            self,
            x,
            y,
            RgbaBlock {
                rgba00: conv(block.rgb00),
                rgba01: conv(block.rgb01),
                rgba10: conv(block.rgb10),
                rgba11: conv(block.rgb11),
            },
        );
    }
}

impl<const REVERSE: bool> RgbaBlockVisitorImpl<f32> for RGBAWriter<'_, REVERSE> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbaBlock<f32>) {
        #[inline(always)]
        unsafe fn write(x: usize, y: usize, width: usize, dst: *mut u8, px: RgbaPixel<f32>) {
            let offset = y * width + x;
            let dst = dst.add(offset * 4);
            dst.cast::<[u8; 4]>().write_unaligned([
                (px.r * 255.0) as u8,
                (px.g * 255.0) as u8,
                (px.b * 255.0) as u8,
                (px.a * 255.0) as u8,
            ]);
        }

        let x = self.window.x + x;
        let y = self.window.y + y;

        write(x, y, self.dst_width, self.dst, block.rgba00);
        write(x + 1, y, self.dst_width, self.dst, block.rgba01);
        write(x, y + 1, self.dst_width, self.dst, block.rgba10);
        write(x + 1, y + 1, self.dst_width, self.dst, block.rgba11);
    }
}

#[cfg(target_arch = "aarch64")]
impl<const REVERSE: bool> RgbaBlockVisitorImpl<float32x4_t> for RGBAWriter<'_, REVERSE> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbaBlock<float32x4_t>) {
        use crate::vector::neon::util::float32x4x2_to_uint8x8_t;
        use std::mem::transmute;

        let x = self.window.x + x;
        let y = self.window.y + y;

        let r0 = float32x4x2_to_uint8x8_t(block.rgba00.r.vmulf(255.0), block.rgba01.r.vmulf(255.0));
        let g0 = float32x4x2_to_uint8x8_t(block.rgba00.g.vmulf(255.0), block.rgba01.g.vmulf(255.0));
        let b0 = float32x4x2_to_uint8x8_t(block.rgba00.b.vmulf(255.0), block.rgba01.b.vmulf(255.0));
        let a0 = float32x4x2_to_uint8x8_t(block.rgba00.a.vmulf(255.0), block.rgba01.a.vmulf(255.0));

        let r1 = float32x4x2_to_uint8x8_t(block.rgba10.r.vmulf(255.0), block.rgba11.r.vmulf(255.0));
        let g1 = float32x4x2_to_uint8x8_t(block.rgba10.g.vmulf(255.0), block.rgba11.g.vmulf(255.0));
        let b1 = float32x4x2_to_uint8x8_t(block.rgba10.b.vmulf(255.0), block.rgba11.b.vmulf(255.0));
        let a1 = float32x4x2_to_uint8x8_t(block.rgba10.a.vmulf(255.0), block.rgba11.a.vmulf(255.0));

        let rgba0 = transmute::<[uint8x8_t; 4], uint8x8x4_t>([r0, g0, b0, a0]);
        let rgba1 = transmute::<[uint8x8_t; 4], uint8x8x4_t>([r1, g1, b1, a1]);

        {
            let offset = y * self.dst_width + x;
            let dst = self.dst.add(offset * 4);
            vst4_u8(dst, rgba0);
        }

        {
            let offset = (y + 1) * self.dst_width + x;
            let dst = self.dst.add(offset * 4);
            vst4_u8(dst, rgba1);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<const REVERSE: bool> RgbaBlockVisitorImpl<__m256> for RGBAWriter<'_, REVERSE> {
    #[target_feature(enable = "avx2")]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbaBlock<__m256>) {
        use crate::vector::avx2::util::pack_f32x8_rgba_u8x32;

        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset00 = y * self.dst_width + x;
        let offset01 = offset00 + 8;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = offset10 + 8;

        #[inline(always)]
        unsafe fn pack_rgba<const REVERSE: bool>(
            r: __m256,
            g: __m256,
            b: __m256,
            a: __m256,
        ) -> [u8; 32] {
            if REVERSE {
                pack_f32x8_rgba_u8x32(b, g, r, a)
            } else {
                pack_f32x8_rgba_u8x32(r, g, b, a)
            }
        }

        let rgba00 = pack_rgba::<REVERSE>(
            block.rgba00.r.vmulf(255.0),
            block.rgba00.g.vmulf(255.0),
            block.rgba00.b.vmulf(255.0),
            block.rgba00.a.vmulf(255.0),
        );
        let rgba01 = pack_rgba::<REVERSE>(
            block.rgba01.r.vmulf(255.0),
            block.rgba01.g.vmulf(255.0),
            block.rgba01.b.vmulf(255.0),
            block.rgba01.a.vmulf(255.0),
        );
        let rgba10 = pack_rgba::<REVERSE>(
            block.rgba10.r.vmulf(255.0),
            block.rgba10.g.vmulf(255.0),
            block.rgba10.b.vmulf(255.0),
            block.rgba10.a.vmulf(255.0),
        );
        let rgba11 = pack_rgba::<REVERSE>(
            block.rgba11.r.vmulf(255.0),
            block.rgba11.g.vmulf(255.0),
            block.rgba11.b.vmulf(255.0),
            block.rgba11.a.vmulf(255.0),
        );

        self.dst
            .add(offset00 * 4)
            .cast::<[u8; 32]>()
            .write_unaligned(rgba00);
        self.dst
            .add(offset01 * 4)
            .cast::<[u8; 32]>()
            .write_unaligned(rgba01);
        self.dst
            .add(offset10 * 4)
            .cast::<[u8; 32]>()
            .write_unaligned(rgba10);
        self.dst
            .add(offset11 * 4)
            .cast::<[u8; 32]>()
            .write_unaligned(rgba11);
    }
}
