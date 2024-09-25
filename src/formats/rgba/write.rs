use super::{RgbaPixel, RgbaSrc};
use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Window};
use std::marker::PhantomData;

pub(crate) struct RgbaWriter<'a, const REVERSE: bool, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    window: Window,
    dst_width: usize,
    dst: *mut P,

    max_value: f32,

    rgba_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, const REVERSE: bool, P, S> RgbaWriter<'a, REVERSE, P, S>
where
    P: PrimitiveInternal,
    S: RgbaSrc,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [P]>,
        bits_per_component: usize,
        window: Option<Window>,
        rgba_src: S,
    ) -> Result<(), ConvertError> {
        if !dst_planes.bounds_check(dst_width, dst_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::RGBA(dst) = dst_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(if REVERSE {
                PixelFormat::BGRA
            } else {
                PixelFormat::RGBA
            }));
        };

        visit(
            dst_width,
            dst_height,
            window,
            Self {
                window: window.unwrap_or(Window {
                    x: 0,
                    y: 0,
                    width: dst_width,
                    height: dst_height,
                }),
                dst_width,
                dst: dst.as_mut_ptr(),
                max_value: crate::formats::max_value_for_bits(bits_per_component),
                rgba_src,
                _m: PhantomData,
            },
        )
    }
}

impl<'a, const REVERSE: bool, B, S> Image2x2Visitor for RgbaWriter<'a, REVERSE, B, S>
where
    B: PrimitiveInternal,
    S: RgbaSrc,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let block = self
            .rgba_src
            .read::<V>(x - self.window.x, y - self.window.y);

        let offset00 = y * self.dst_width + x;
        let offset10 = (y + 1) * self.dst_width + x;

        B::write_interleaved_4x_2x(
            self.dst.add(offset00 * 4),
            [
                multiply_and_reverse::<REVERSE, V>(block.px00, self.max_value),
                multiply_and_reverse::<REVERSE, V>(block.px01, self.max_value),
            ],
        );

        B::write_interleaved_4x_2x(
            self.dst.add(offset10 * 4),
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
