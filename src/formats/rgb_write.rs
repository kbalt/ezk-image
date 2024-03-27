use super::rgb::{RgbBlock, RgbBlockVisitorImpl, RgbPixel};
use super::rgba::{RgbaBlock, RgbaBlockVisitorImpl, RgbaPixel};
use crate::vector::Vector;
use crate::{arch::*, RawMutSliceU8, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RGBWriter<'a, const REVERSE: bool = false> {
    window: Rect,

    dst_width: usize,
    dst: *mut u8,

    _m: PhantomData<&'a mut [u8]>,
}

impl<'a, const REVERSE: bool> RGBWriter<'a, REVERSE> {
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

        assert!(dst_width * dst_height * 3 <= dst.len());
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

impl<const REVERSE: bool, V: Vector> RgbaBlockVisitorImpl<V> for RGBWriter<'_, REVERSE>
where
    Self: RgbBlockVisitorImpl<V>,
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbaBlock<V>) {
        unsafe fn conv<V: Vector>(px: RgbaPixel<V>) -> RgbPixel<V> {
            RgbPixel {
                r: px.r,
                g: px.g,
                b: px.b,
            }
        }

        RgbBlockVisitorImpl::visit(
            self,
            x,
            y,
            RgbBlock {
                rgb00: conv(block.rgba00),
                rgb01: conv(block.rgba01),
                rgb10: conv(block.rgba10),
                rgb11: conv(block.rgba11),
            },
        );
    }
}
impl<const REVERSE: bool> RgbBlockVisitorImpl<f32> for RGBWriter<'_, REVERSE> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<f32>) {
        #[inline(always)]
        unsafe fn write(x: usize, y: usize, width: usize, dst: *mut u8, px: RgbPixel<f32>) {
            let offset = y * width + x;
            let dst = dst.add(offset * 3);
            dst.cast::<[u8; 3]>()
                .write_unaligned([px.r as u8, px.g as u8, px.b as u8]);
        }

        let x = self.window.x + x;
        let y = self.window.y + y;

        write(x, y, self.dst_width, self.dst, block.rgb00);
        write(x + 1, y, self.dst_width, self.dst, block.rgb01);
        write(x, y + 1, self.dst_width, self.dst, block.rgb10);
        write(x + 1, y + 1, self.dst_width, self.dst, block.rgb11);
    }
}

#[cfg(target_arch = "aarch64")]
impl<const REVERSE: bool> RgbBlockVisitorImpl<float32x4_t> for RGBWriter<'_, REVERSE> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<float32x4_t>) {
        use crate::vector::neon::util::float32x4x2_to_uint8x8_t;
        use std::mem::transmute;

        let x = self.window.x + x;
        let y = self.window.y + y;

        let r0 = float32x4x2_to_uint8x8_t(block.rgb00.r.vmulf(255.0), block.rgb01.r.vmulf(255.0));
        let g0 = float32x4x2_to_uint8x8_t(block.rgb00.g.vmulf(255.0), block.rgb01.g.vmulf(255.0));
        let b0 = float32x4x2_to_uint8x8_t(block.rgb00.b.vmulf(255.0), block.rgb01.b.vmulf(255.0));

        let r1 = float32x4x2_to_uint8x8_t(block.rgb10.r.vmulf(255.0), block.rgb11.r.vmulf(255.0));
        let g1 = float32x4x2_to_uint8x8_t(block.rgb10.g.vmulf(255.0), block.rgb11.g.vmulf(255.0));
        let b1 = float32x4x2_to_uint8x8_t(block.rgb10.b.vmulf(255.0), block.rgb11.b.vmulf(255.0));

        let rgb0 = transmute::<[uint8x8_t; 3], uint8x8x3_t>([r0, g0, b0]);
        let rgb1 = transmute::<[uint8x8_t; 3], uint8x8x3_t>([r1, g1, b1]);

        {
            let offset = y * self.dst_width + x;
            let dst = self.dst.add(offset * 3);
            vst3_u8(dst, rgb0);
        }

        {
            let offset = (y + 1) * self.dst_width + x;
            let dst = self.dst.add(offset * 3);
            vst3_u8(dst, rgb1);
        }
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<const REVERSE: bool> RgbBlockVisitorImpl<__m256> for RGBWriter<'_, REVERSE> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<__m256>) {
        use crate::vector::avx2::util::packf32x8_rgb_u8x24;

        #[inline(always)]
        unsafe fn pack_rgb<const REVERSE: bool>(r: __m256, g: __m256, b: __m256) -> [u8; 24] {
            if REVERSE {
                packf32x8_rgb_u8x24(b, g, r)
            } else {
                packf32x8_rgb_u8x24(r, g, b)
            }
        }

        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset00 = y * self.dst_width + x;
        let offset01 = offset00 + 8;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = offset10 + 8;

        let rgb00 = pack_rgb::<REVERSE>(
            block.rgb00.r.vmulf(255.0),
            block.rgb00.g.vmulf(255.0),
            block.rgb00.b.vmulf(255.0),
        );
        let rgb01 = pack_rgb::<REVERSE>(
            block.rgb01.r.vmulf(255.0),
            block.rgb01.g.vmulf(255.0),
            block.rgb01.b.vmulf(255.0),
        );
        let rgb10 = pack_rgb::<REVERSE>(
            block.rgb10.r.vmulf(255.0),
            block.rgb10.g.vmulf(255.0),
            block.rgb10.b.vmulf(255.0),
        );
        let rgb11 = pack_rgb::<REVERSE>(
            block.rgb11.r.vmulf(255.0),
            block.rgb11.g.vmulf(255.0),
            block.rgb11.b.vmulf(255.0),
        );

        self.dst
            .add(offset00 * 3)
            .cast::<[u8; 24]>()
            .write_unaligned(rgb00);

        self.dst
            .add(offset01 * 3)
            .cast::<[u8; 24]>()
            .write_unaligned(rgb01);

        self.dst
            .add(offset10 * 3)
            .cast::<[u8; 24]>()
            .write_unaligned(rgb10);

        self.dst
            .add(offset11 * 3)
            .cast::<[u8; 24]>()
            .write_unaligned(rgb11);
    }
}
