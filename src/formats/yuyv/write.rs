#![allow(clippy::too_many_arguments)]

use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I422Block, I422Src, PixelFormat, PixelFormatPlanes, Window};
use std::marker::PhantomData;

pub(crate) struct YUYVWriter<'a, P, S>
where
    P: PrimitiveInternal,
    S: I422Src,
{
    window: Window,
    dst_width: usize,
    dst_yuyv: *mut P,
    max_value: f32,

    i422_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> YUYVWriter<'a, P, S>
where
    P: PrimitiveInternal,
    S: I422Src,
{
    pub(crate) fn write(
        dst_width: usize,
        dst_height: usize,
        dst_planes: PixelFormatPlanes<&'a mut [P]>,
        bits_per_component: usize,
        window: Option<Window>,
        i422_src: S,
    ) -> Result<(), ConvertError> {
        if !dst_planes.bounds_check(dst_width, dst_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::YUYV(yuyv) = dst_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::YUYV));
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
                dst_yuyv: yuyv.as_mut_ptr(),
                max_value: crate::formats::max_value_for_bits(bits_per_component),
                i422_src,
                _m: PhantomData,
            },
        )
    }

    unsafe fn write_yuyv<V: Vector>(&mut self, y: V, uv: V, offset0: usize)
    where
        P: PrimitiveInternal,
    {
        let (yuyv00, yuyv01) = y.zip(uv);

        P::write_2x(self.dst_yuyv.add(offset0), yuyv00, yuyv01);
    }
}

impl<P, S> Image2x2Visitor for YUYVWriter<'_, P, S>
where
    P: PrimitiveInternal,
    S: I422Src,
{
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize) {
        let I422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        } = self
            .i422_src
            .read::<V>(x - self.window.x, y - self.window.y);

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);
        let u0 = u0.vmulf(self.max_value);
        let u1 = u1.vmulf(self.max_value);
        let v0 = v0.vmulf(self.max_value);
        let v1 = v1.vmulf(self.max_value);

        let offset0 = (y * self.dst_width) + x;
        let offset1 = ((y + 1) * self.dst_width) + x;

        let (uv00, uv01) = u0.zip(v0);
        let (uv10, uv11) = u1.zip(v1);

        self.write_yuyv(y00, uv00, offset0 * 2);
        self.write_yuyv(y01, uv01, (offset0 + V::LEN) * 2);
        self.write_yuyv(y10, uv10, offset1 * 2);
        self.write_yuyv(y11, uv11, (offset1 + V::LEN) * 2);
    }
}
