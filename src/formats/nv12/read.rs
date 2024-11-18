use crate::formats::{I420Block, I420Src};
use crate::planes::read_planes;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct NV12Reader<'a, P: PrimitiveInternal> {
    y: *const u8,
    uv: *const u8,

    y_stride: usize,
    uv_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> NV12Reader<'a, P> {
    pub(crate) fn new(src: &'a dyn ImageRef) -> Result<Self, ConvertError> {
        src.bounds_check()?;

        let [(y, y_stride), (uv, uv_stride)] = read_planes(src.planes())?;

        Ok(Self {
            y: y.as_ptr(),
            uv: uv.as_ptr(),
            y_stride,
            uv_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }
}

impl<P: PrimitiveInternal> I420Src for NV12Reader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let y00_offset = y * self.y_stride + x * P::SIZE;
        let y10_offset = (y + 1) * self.y_stride + x * P::SIZE;

        let uv_offset = (y / 2) * (self.uv_stride) + x * P::SIZE;

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(y00_offset));
        let y01 = P::load::<V>(self.y.add(y00_offset + V::LEN * P::SIZE));
        let y10 = P::load::<V>(self.y.add(y10_offset));
        let y11 = P::load::<V>(self.y.add(y10_offset + V::LEN * P::SIZE));

        // Load U and V
        let uv0 = P::load::<V>(self.uv.add(uv_offset));
        let uv1 = P::load::<V>(self.uv.add(uv_offset + V::LEN * P::SIZE));

        let (u, v) = uv0.unzip(uv1);

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u = u.vdivf(self.max_value);
        let v = v.vdivf(self.max_value);

        I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        }
    }
}
