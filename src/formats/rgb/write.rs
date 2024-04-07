use super::{RgbPixel, RgbSrc};
use crate::bits::BitsInternal;
use crate::formats::reader::{read, ImageReader};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct RgbWriter<const REVERSE: bool, B, S>
where
    B: BitsInternal,
    S: RgbSrc,
{
    dst_width: usize,
    dst: *mut B::Primitive,

    max_value: f32,

    rgb_src: S,

    _b: PhantomData<B>,
}

impl<const REVERSE: bool, B, S> RgbWriter<REVERSE, B, S>
where
    B: BitsInternal,
    S: RgbSrc,
{
    pub(crate) fn read(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&mut [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
        rgb_src: S,
    ) {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::RGB(dst) = dst_planes else {
            panic!("Invalid PixelFormatPlanes for RgbWriter");
        };

        read(
            dst_width,
            dst_height,
            window,
            Self {
                dst_width,
                dst: dst.as_mut_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                rgb_src,
                _b: PhantomData,
            },
        )
    }
}

impl<const REVERSE: bool, B, S> ImageReader for RgbWriter<REVERSE, B, S>
where
    B: BitsInternal,
    S: RgbSrc,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.rgb_src.read::<V>(x, y);

        let offset00 = y * self.dst_width + x;
        let offset10 = (y + 1) * self.dst_width + x;

        B::write_interleaved_3x_2x(
            self.dst.add(offset00 * 3),
            [
                multiply_and_reverse::<REVERSE, V>(block.rgb00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.rgb01, self.max_value),
            ],
        );

        B::write_interleaved_3x_2x(
            self.dst.add(offset10 * 3),
            [
                multiply_and_reverse::<REVERSE, V>(block.rgb10, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.rgb11, self.max_value),
            ],
        );
    }
}

#[inline(always)]
unsafe fn multiply_and_reverse<const REVERSE: bool, V: Vector>(
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
