use super::{RgbaPixel, RgbaSrc};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct RgbaWriter<'a, const REVERSE: bool, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    rgba: *mut u8,

    rgba_stride: usize,

    max_value: f32,

    rgba_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, const REVERSE: bool, P, S> RgbaWriter<'a, REVERSE, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    pub(crate) fn write(dst: &'a mut dyn ImageMut, rgba_src: S) -> Result<(), ConvertError> {
        dst.bounds_check()?;

        let dst_width = dst.width();
        let dst_height = dst.height();
        let dst_format = dst.format();

        let [(rgba, rgba_stride)] = read_planes_mut(dst.planes_mut())?;

        visit(
            dst_width,
            dst_height,
            Self {
                rgba: rgba.as_mut_ptr(),
                rgba_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
                rgba_src,
                _m: PhantomData,
            },
        )
    }
}

impl<const REVERSE: bool, P, S> Image2x2Visitor for RgbaWriter<'_, REVERSE, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self.rgba_src.read::<V>(x, y);

        let offset00 = y * self.rgba_stride + x * 4 * P::SIZE;
        let offset10 = (y + 1) * self.rgba_stride + x * 4 * P::SIZE;

        P::write_interleaved_4x_2x(
            self.rgba.add(offset00),
            [
                multiply_and_reverse::<REVERSE, V>(block.px00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.px01, self.max_value),
            ],
        );

        P::write_interleaved_4x_2x(
            self.rgba.add(offset10),
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
