use super::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::bits::BitsInternal;
use crate::formats::rgb::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RGBAWriter<'a, const REVERSE: bool, B: BitsInternal> {
    window: Rect,

    dst_width: usize,
    dst: *mut B::Primitive,
    max_value: f32,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, const REVERSE: bool, B: BitsInternal> RGBAWriter<'a, REVERSE, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
        bits_per_channel: usize,
        window: Option<Rect>,
    ) -> Self {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::RGBA(dst) = dst_planes else {
            panic!("Invalid PixelFormatPlanes for RGBAWriter");
        };

        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: dst_width,
            height: dst_height,
        });

        assert!((window.x + window.width) <= dst_width);
        assert!((window.y + window.height) <= dst_height);

        Self {
            window,
            dst_width,
            dst: dst.as_mut_ptr(),
            max_value: crate::max_value_for_bits(bits_per_channel),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<const REVERSE: bool, B> RgbBlockVisitor for RGBAWriter<'_, REVERSE, B>
where
    Self: RgbaBlockVisitor,
    B: BitsInternal,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        #[inline(always)]
        unsafe fn conv<V: Vector>(px: RgbPixel<V>) -> RgbaPixel<V> {
            RgbaPixel {
                r: px.r,
                g: px.g,
                b: px.b,
                a: V::splat(1.0),
            }
        }

        RgbaBlockVisitor::visit(
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

impl<const REVERSE: bool, B: BitsInternal> RgbaBlockVisitor for RGBAWriter<'_, REVERSE, B> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbaBlock<V>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset00 = y * self.dst_width + x;
        let offset10 = (y + 1) * self.dst_width + x;

        B::write_interleaved_4x_2x(
            self.dst.add(offset00 * 4),
            [
                multiply_and_reverse::<REVERSE, V>(block.rgba00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.rgba01, self.max_value),
            ],
        );

        B::write_interleaved_4x_2x(
            self.dst.add(offset10 * 4),
            [
                multiply_and_reverse::<REVERSE, V>(block.rgba10, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.rgba11, self.max_value),
            ],
        );
    }
}

#[inline(always)]
unsafe fn multiply_and_reverse<const REVERSE: bool, V: Vector>(
    px: RgbaPixel<V>,
    max_value: f32,
) -> [V; 4] {
    let r = px.r.vmulf(max_value);
    let g = px.g.vmulf(max_value);
    let b = px.b.vmulf(max_value);
    let a = px.a.vmulf(max_value);

    if REVERSE {
        [b, g, r, a]
    } else {
        [r, g, b, a]
    }
}
