use super::rgb::{RgbBlock, RgbBlockVisitorImpl, RgbPixel};
use super::rgba::{RgbaBlock, RgbaBlockVisitorImpl, RgbaPixel};
use crate::bits::{Bits, U8};
use crate::endian::Endian;
use crate::vector::Vector;
use crate::{arch::*, RawMutSliceU8, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RGBWriter<'a, const REVERSE: bool, B> {
    window: Rect,

    dst_width: usize,
    dst: *mut u8,
    max_value: f32,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, const REVERSE: bool, B: Bits> RGBWriter<'a, REVERSE, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst: RawMutSliceU8<'a>,
        bits_per_channel: usize,
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
            max_value: crate::max_value_for_bits(bits_per_channel),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<const REVERSE: bool, V, B> RgbaBlockVisitorImpl<V> for RGBWriter<'_, REVERSE, B>
where
    Self: RgbBlockVisitorImpl<V>,
    V: Vector,
    B: Bits,
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
impl<const REVERSE: bool, B: Bits> RgbBlockVisitorImpl<f32> for RGBWriter<'_, REVERSE, B> {
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<f32>) {
        #[inline(always)]
        unsafe fn write<B: Bits>(
            x: usize,
            y: usize,
            width: usize,
            dst: *mut B::Primitive,
            px: RgbPixel<f32>,
        ) {
            let offset = y * width + x;
            let dst = dst.add(offset * 3);
            dst.cast::<[B::Primitive; 3]>().write_unaligned([
                B::primitive_from_f32(px.r),
                B::primitive_from_f32(px.g),
                B::primitive_from_f32(px.b),
            ]);
        }

        let x = self.window.x + x;
        let y = self.window.y + y;

        let dst = self.dst.cast::<B::Primitive>();

        write::<B>(x, y, self.dst_width, dst, block.rgb00);
        write::<B>(x + 1, y, self.dst_width, dst, block.rgb01);
        write::<B>(x, y + 1, self.dst_width, dst, block.rgb10);
        write::<B>(x + 1, y + 1, self.dst_width, dst, block.rgb11);
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
impl<const REVERSE: bool> RgbBlockVisitorImpl<__m256> for RGBWriter<'_, REVERSE, U8> {
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
            block.rgb00.r.vmulf(self.max_value),
            block.rgb00.g.vmulf(self.max_value),
            block.rgb00.b.vmulf(self.max_value),
        );
        let rgb01 = pack_rgb::<REVERSE>(
            block.rgb01.r.vmulf(self.max_value),
            block.rgb01.g.vmulf(self.max_value),
            block.rgb01.b.vmulf(self.max_value),
        );
        let rgb10 = pack_rgb::<REVERSE>(
            block.rgb10.r.vmulf(self.max_value),
            block.rgb10.g.vmulf(self.max_value),
            block.rgb10.b.vmulf(self.max_value),
        );
        let rgb11 = pack_rgb::<REVERSE>(
            block.rgb11.r.vmulf(self.max_value),
            block.rgb11.g.vmulf(self.max_value),
            block.rgb11.b.vmulf(self.max_value),
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl<const REVERSE: bool, B: Bits<Primitive = u16>> RgbBlockVisitorImpl<__m256>
    for RGBWriter<'_, REVERSE, B>
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<__m256>) {
        use crate::vector::avx2::util::packf32x8_rgb_u16x24;

        #[inline(always)]
        unsafe fn pack_rgb<const REVERSE: bool, E: Endian>(
            r: __m256,
            g: __m256,
            b: __m256,
        ) -> [u16; 24] {
            if REVERSE {
                packf32x8_rgb_u16x24::<E>(b, g, r)
            } else {
                packf32x8_rgb_u16x24::<E>(r, g, b)
            }
        }

        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset00 = y * self.dst_width + x;
        let offset01 = offset00 + 8;
        let offset10 = (y + 1) * self.dst_width + x;
        let offset11 = offset10 + 8;

        let rgb00 = pack_rgb::<REVERSE, B::Endian>(
            block.rgb00.r.vmulf(self.max_value),
            block.rgb00.g.vmulf(self.max_value),
            block.rgb00.b.vmulf(self.max_value),
        );
        let rgb01 = pack_rgb::<REVERSE, B::Endian>(
            block.rgb01.r.vmulf(self.max_value),
            block.rgb01.g.vmulf(self.max_value),
            block.rgb01.b.vmulf(self.max_value),
        );
        let rgb10 = pack_rgb::<REVERSE, B::Endian>(
            block.rgb10.r.vmulf(self.max_value),
            block.rgb10.g.vmulf(self.max_value),
            block.rgb10.b.vmulf(self.max_value),
        );
        let rgb11 = pack_rgb::<REVERSE, B::Endian>(
            block.rgb11.r.vmulf(self.max_value),
            block.rgb11.g.vmulf(self.max_value),
            block.rgb11.b.vmulf(self.max_value),
        );

        self.dst
            .add(offset00 * 3)
            .cast::<[u16; 24]>()
            .write_unaligned(rgb00);

        self.dst
            .add(offset01 * 3)
            .cast::<[u16; 24]>()
            .write_unaligned(rgb01);

        self.dst
            .add(offset10 * 3)
            .cast::<[u16; 24]>()
            .write_unaligned(rgb10);

        self.dst
            .add(offset11 * 3)
            .cast::<[u16; 24]>()
            .write_unaligned(rgb11);
    }
}
