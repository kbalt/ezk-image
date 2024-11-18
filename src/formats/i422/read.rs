use super::{I422Block, I422Src};
use crate::planes::read_planes;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct I422Reader<'a, P: PrimitiveInternal> {
    y: *const u8,
    u: *const u8,
    v: *const u8,

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> I422Reader<'a, P> {
    pub(crate) fn new(src: &'a dyn ImageRef) -> Result<Self, ConvertError> {
        src.bounds_check()?;

        let [(y, y_stride), (u, u_stride), (v, v_stride)] = read_planes(src.planes())?;

        Ok(Self {
            y: y.as_ptr(),
            u: u.as_ptr(),
            v: v.as_ptr(),
            y_stride,
            u_stride,
            v_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }
}

impl<P: PrimitiveInternal> I422Src for I422Reader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V> {
        let y00_offset = (y * self.y_stride) + x * P::SIZE;
        let y10_offset = ((y + 1) * self.y_stride) + x * P::SIZE;

        let u0_offset = y * self.u_stride + (x / 2) * P::SIZE;
        let u1_offset = (y + 1) * self.u_stride + (x / 2) * P::SIZE;

        let v0_offset = y * self.v_stride + (x / 2) * P::SIZE;
        let v1_offset = (y + 1) * self.v_stride + (x / 2) * P::SIZE;

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(y00_offset));
        let y01 = P::load::<V>(self.y.add(y00_offset + V::LEN * P::SIZE));
        let y10 = P::load::<V>(self.y.add(y10_offset));
        let y11 = P::load::<V>(self.y.add(y10_offset + V::LEN * P::SIZE));

        // Load U and V
        let u0 = P::load::<V>(self.u.add(u0_offset));
        let u1 = P::load::<V>(self.u.add(u1_offset));

        let v0 = P::load::<V>(self.v.add(v0_offset));
        let v1 = P::load::<V>(self.v.add(v1_offset));

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
