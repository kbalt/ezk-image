use crate::bits::BitsInternal;
use crate::formats::reader::{visit, ImageVisitor};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect, RgbaPixel, RgbaSrc};
use std::marker::PhantomData;

pub(crate) struct RgbWriter<'a, const REVERSE: bool, B, S>
where
    B: BitsInternal,
    S: RgbaSrc,
{
    dst_width: usize,
    dst: *mut B::Primitive,

    max_value: f32,

    rgba_src: S,

    _m: PhantomData<&'a mut [B::Primitive]>,
}

impl<'a, const REVERSE: bool, B, S> RgbWriter<'a, REVERSE, B, S>
where
    B: BitsInternal,
    S: RgbaSrc,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&mut [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
        rgba_src: S,
    ) {
        assert!(dst_planes.bounds_check(dst_width, dst_height));

        let PixelFormatPlanes::RGB(dst) = dst_planes else {
            panic!("Invalid PixelFormatPlanes for RgbWriter");
        };

        visit(
            dst_width,
            dst_height,
            window,
            Self {
                dst_width,
                dst: dst.as_mut_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                rgba_src,
                _m: PhantomData,
            },
        )
    }
}

impl<'a, const REVERSE: bool, B, S> ImageVisitor for RgbWriter<'a, REVERSE, B, S>
where
    B: BitsInternal,
    S: RgbaSrc,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.rgba_src.read::<V>(x, y);

        let offset00 = y * self.dst_width + x;
        let offset10 = (y + 1) * self.dst_width + x;

        B::write_interleaved_3x_2x(
            self.dst.add(offset00 * 3),
            [
                multiply_and_reverse::<REVERSE, V>(block.px00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.px01, self.max_value),
            ],
        );

        B::write_interleaved_3x_2x(
            self.dst.add(offset10 * 3),
            [
                multiply_and_reverse::<REVERSE, V>(block.px10, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.px11, self.max_value),
            ],
        );
    }
}

#[inline(always)]
unsafe fn multiply_and_reverse<const REVERSE: bool, V: Vector>(
    px: RgbaPixel<V>,
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
