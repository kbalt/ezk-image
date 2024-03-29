use super::{RgbBlock, RgbBlockVisitorImpl, RgbPixel};
use crate::bits::BitsInternal;
use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitorImpl, RgbaPixel};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

/// Writes 3 Bytes for every visited pixel in R G B order
pub(crate) struct RGBWriter<'a, const REVERSE: bool, B: BitsInternal> {
    window: Rect,

    dst_width: usize,
    dst: *mut B::Primitive,
    max_value: f32,

    _m: PhantomData<&'a mut [u8]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, const REVERSE: bool, B: BitsInternal> RGBWriter<'a, REVERSE, B> {
    pub(crate) fn new(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
        bits_per_channel: usize,
        window: Option<Rect>,
    ) -> Self {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::RGB(dst) = dst_planes else {
            panic!("Invalid PixelFormatPlanes for RGBWriter");
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

impl<const REVERSE: bool, V, B> RgbaBlockVisitorImpl<V> for RGBWriter<'_, REVERSE, B>
where
    Self: RgbBlockVisitorImpl<V>,
    V: Vector,
    B: BitsInternal,
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

impl<const REVERSE: bool, V: Vector, B: BitsInternal> RgbBlockVisitorImpl<V>
    for RGBWriter<'_, REVERSE, B>
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let offset00 = y * self.dst_width + x;
        let offset10 = (y + 1) * self.dst_width + x;

        B::write_interleaved_3x_2x(
            self.dst.add(offset00 * 3),
            [
                divide_and_reverse::<REVERSE, _>(block.rgb00, self.max_value),
                divide_and_reverse::<REVERSE, _>(block.rgb01, self.max_value),
            ],
        );

        B::write_interleaved_3x_2x(
            self.dst.add(offset10 * 3),
            [
                divide_and_reverse::<REVERSE, _>(block.rgb10, self.max_value),
                divide_and_reverse::<REVERSE, _>(block.rgb11, self.max_value),
            ],
        );
    }
}

#[inline(always)]
unsafe fn divide_and_reverse<const REVERSE: bool, V: Vector>(
    px: RgbPixel<V>,
    max_value: f32,
) -> [V; 3] {
    let r = px.r.vmulf(max_value);
    let g = px.g.vmulf(max_value);
    let b = px.b.vmulf(max_value);

    if REVERSE {
        [b, g, r]
    } else {
        [r, g, b]
    }
}
