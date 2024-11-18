use crate::planes::read_planes;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I422Block, I422Src, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct YUYVReader<'a, P: PrimitiveInternal> {
    yuyv: *const u8,

    yuyv_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> YUYVReader<'a, P> {
    pub(crate) fn new(src: &'a dyn ImageRef) -> Result<Self, ConvertError> {
        src.bounds_check()?;

        let [(yuyv, yuyv_stride)] = read_planes(src.planes())?;

        Ok(Self {
            yuyv: yuyv.as_ptr(),
            yuyv_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }

    unsafe fn read_yuyv<V: Vector>(&mut self, offset: usize) -> (V, V) {
        let yuyv00 = P::load::<V>(self.yuyv.add(offset));
        let yuyv01 = P::load::<V>(self.yuyv.add(offset + V::LEN * P::SIZE));

        yuyv00.unzip(yuyv01)
    }
}

impl<P: PrimitiveInternal> I422Src for YUYVReader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V> {
        let offset0 = y * self.yuyv_stride + x * 2 * P::SIZE;
        let offset1 = (y + 1) * self.yuyv_stride + x * 2 * P::SIZE;

        let (y00, uv00) = self.read_yuyv::<V>(offset0);
        let (y01, uv01) = self.read_yuyv::<V>(offset0 + V::LEN * 2 * P::SIZE);
        let (y10, uv10) = self.read_yuyv::<V>(offset1);
        let (y11, uv11) = self.read_yuyv::<V>(offset1 + V::LEN * 2 * P::SIZE);

        let (u0, v0) = uv00.unzip(uv01);
        let (u1, v1) = uv10.unzip(uv11);

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u0 = u0.vdivf(self.max_value);
        let u1 = u1.vdivf(self.max_value);

        let v0 = v0.vdivf(self.max_value);
        let v1 = v1.vdivf(self.max_value);

        I422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        }
    }
}
