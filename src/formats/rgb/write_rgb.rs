use crate::formats::rgb::{RgbaPixel, RgbaSrc, SWIZZLE_BGRA, SWIZZLE_RGBA};
use crate::formats::visit_2x2::{Image2x2Visitor, visit};
use crate::planes::read_planes_mut;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct WriteRgb<'a, const SWIZZLE: u8, P, S>
where
    P: Primitive,
    S: RgbaSrc,
{
    rgb: &'a mut [u8],

    rgb_stride: usize,

    max_value: f32,

    rgba_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, const SWIZZLE: u8, P, S> WriteRgb<'a, SWIZZLE, P, S>
where
    P: Primitive,
    S: RgbaSrc,
{
    pub(crate) fn write(dst: &'a mut dyn ImageMut, rgba_src: S) -> Result<(), ConvertError> {
        dst.bounds_check()?;

        let dst_width = dst.width();
        let dst_height = dst.height();
        let dst_format = dst.format();

        let [(rgb, rgb_stride)] = read_planes_mut(dst.planes_mut())?;

        visit(
            dst_width,
            dst_height,
            Self {
                rgb,
                rgb_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                rgba_src,
                _m: PhantomData,
            },
        );

        Ok(())
    }
}

impl<const SWIZZLE: u8, P, S> Image2x2Visitor for WriteRgb<'_, SWIZZLE, P, S>
where
    P: Primitive,
    S: RgbaSrc,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.rgba_src.read::<V>(x, y);

        let offset00 = y * self.rgb_stride + x * 3 * P::SIZE;
        let offset10 = (y + 1) * self.rgb_stride + x * 3 * P::SIZE;

        P::write_interleaved_3x_2x(
            &mut self.rgb[offset00..],
            [
                multiply_and_reverse::<SWIZZLE, V>(block.px00, self.max_value),
                multiply_and_reverse::<SWIZZLE, V>(block.px01, self.max_value),
            ],
        );

        P::write_interleaved_3x_2x(
            &mut self.rgb[offset10..],
            [
                multiply_and_reverse::<SWIZZLE, V>(block.px10, self.max_value),
                multiply_and_reverse::<SWIZZLE, V>(block.px11, self.max_value),
            ],
        );
    }
}

#[inline(always)]
unsafe fn multiply_and_reverse<const SWIZZLE: u8, V: Vector>(
    px: RgbaPixel<V>,
    max_value: f32,
) -> [V; 3] {
    let r = px.r.vmulf(max_value);
    let g = px.g.vmulf(max_value);
    let b = px.b.vmulf(max_value);

    match SWIZZLE {
        SWIZZLE_RGBA => [r, g, b],
        SWIZZLE_BGRA => [b, g, r],
        _ => unreachable!("unknown swizzle {SWIZZLE}"),
    }
}
