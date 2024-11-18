use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt, RgbaPixel, RgbaSrc};
use std::marker::PhantomData;

pub(crate) struct RgbWriter<'a, const REVERSE: bool, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    rgb: *mut u8,

    rgb_stride: usize,

    max_value: f32,

    rgba_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, const REVERSE: bool, P, S> RgbWriter<'a, REVERSE, P, S>
where
    P: PrimitiveInternal,
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
                rgb: rgb.as_mut_ptr(),
                rgb_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                rgba_src,
                _m: PhantomData,
            },
        )
    }
}

impl<const REVERSE: bool, P, S> Image2x2Visitor for RgbWriter<'_, REVERSE, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.rgba_src.read::<V>(x, y);

        let offset00 = y * self.rgb_stride + x * 3 * P::SIZE;
        let offset10 = (y + 1) * self.rgb_stride + x * 3 * P::SIZE;

        P::write_interleaved_3x_2x(
            self.rgb.add(offset00),
            [
                multiply_and_reverse::<REVERSE, V>(block.px00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.px01, self.max_value),
            ],
        );

        P::write_interleaved_3x_2x(
            self.rgb.add(offset10),
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
