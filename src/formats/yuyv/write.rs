use crate::formats::visit_2x2::{visit, Image2x2Visitor};
use crate::planes::read_planes_mut;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I422Block, I422Src, ImageMut, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct YUYVWriter<'a, P, S>
where
    P: PrimitiveInternal,
    S: I422Src,
{
    yuyv: *mut u8,

    yuyv_stride: usize,

    max_value: f32,

    i422_src: S,

    _m: PhantomData<&'a mut [P]>,
}

impl<'a, P, S> YUYVWriter<'a, P, S>
where
    P: PrimitiveInternal,
    S: I422Src,
{
    pub(crate) fn write(dst: &'a mut impl ImageMut, i422_src: S) -> Result<(), ConvertError> {
        dst.bounds_check()?;

        let dst_width = dst.width();
        let dst_height = dst.height();
        let dst_format = dst.format();

        let [(yuyv, yuyv_stride)] = read_planes_mut(dst.planes_mut())?;

        visit(
            dst_width,
            dst_height,
            Self {
                yuyv: yuyv.as_mut_ptr(),
                yuyv_stride,
                max_value: crate::formats::max_value_for_bits(dst_format.bits_per_component()),
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

        P::write_2x(self.yuyv.add(offset0), yuyv00, yuyv01);
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
        } = self.i422_src.read::<V>(x, y);

        let y00 = y00.vmulf(self.max_value);
        let y01 = y01.vmulf(self.max_value);
        let y10 = y10.vmulf(self.max_value);
        let y11 = y11.vmulf(self.max_value);
        let u0 = u0.vmulf(self.max_value);
        let u1 = u1.vmulf(self.max_value);
        let v0 = v0.vmulf(self.max_value);
        let v1 = v1.vmulf(self.max_value);

        let offset0 = y * self.yuyv_stride + x * 2 * P::SIZE;
        let offset1 = (y + 1) * self.yuyv_stride + x * 2 * P::SIZE;

        let (uv00, uv01) = u0.zip(v0);
        let (uv10, uv11) = u1.zip(v1);

        self.write_yuyv(y00, uv00, offset0);
        self.write_yuyv(y01, uv01, offset0 + V::LEN * 2 * P::SIZE);
        self.write_yuyv(y10, uv10, offset1);
        self.write_yuyv(y11, uv11, offset1 + V::LEN * 2 * P::SIZE);
    }
}
