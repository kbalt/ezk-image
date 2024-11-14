use super::{I420Block, I420Src};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef};
use std::marker::PhantomData;

pub(crate) struct I420Reader<'a, P: PrimitiveInternal> {
    y: *const u8,
    u: *const u8,
    v: *const u8,

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> I420Reader<'a, P> {
    pub(crate) fn new(src: impl ImageRef<'a>) -> Result<Self, ConvertError> {
        if !src.bounds_check() {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let [y, u, v] = src.planes() else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(src.format()));
        };

        let [y_stride, u_stride, v_stride] = *src.strides() else {
            return Err(ConvertError::InvalidStridesForPixelFormat(src.format()));
        };

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

impl<'a, P: PrimitiveInternal> I420Src for I420Reader<'a, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let y00_offset = (y * self.y_stride) + x;
        let y10_offset = ((y + 1) * self.y_stride) + x;

        let u_offset = (y / 2) * (self.u_stride) + (x / 2);
        let v_offset = (y / 2) * (self.v_stride) + (x / 2);

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(y00_offset));
        let y01 = P::load::<V>(self.y.add(y00_offset + V::LEN));
        let y10 = P::load::<V>(self.y.add(y10_offset));
        let y11 = P::load::<V>(self.y.add(y10_offset + V::LEN));

        // Load U and V
        let u = P::load::<V>(self.u.add(u_offset));
        let v = P::load::<V>(self.v.add(v_offset));

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
