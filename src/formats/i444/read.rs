use super::{I444Block, I444Src};
use crate::planes::read_planes;
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I444Pixel, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct I444Reader<'a, P: PrimitiveInternal> {
    y: *const u8,
    u: *const u8,
    v: *const u8,

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> I444Reader<'a, P> {
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

impl<P: PrimitiveInternal> I444Src for I444Reader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I444Block<V> {
        let y00_offset = (y * self.y_stride) + x * P::SIZE;
        let y10_offset = ((y + 1) * self.y_stride) + x * P::SIZE;

        let u00_offset = (y * self.u_stride) + x * P::SIZE;
        let u10_offset = ((y + 1) * self.y_stride) + x * P::SIZE;

        let v00_offset = (y * self.v_stride) + x * P::SIZE;
        let v10_offset = ((y + 1) * self.y_stride) + x * P::SIZE;

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(y00_offset));
        let y01 = P::load::<V>(self.y.add(y00_offset + V::LEN * P::SIZE));
        let y10 = P::load::<V>(self.y.add(y10_offset));
        let y11 = P::load::<V>(self.y.add(y10_offset + V::LEN * P::SIZE));

        // Load U pixels
        let u00 = P::load::<V>(self.u.add(u00_offset));
        let u01 = P::load::<V>(self.u.add(u00_offset + V::LEN * P::SIZE));
        let u10 = P::load::<V>(self.u.add(u10_offset));
        let u11 = P::load::<V>(self.u.add(u10_offset + V::LEN * P::SIZE));

        // Load V pixels
        let v00 = P::load::<V>(self.v.add(v00_offset));
        let v01 = P::load::<V>(self.v.add(v00_offset + V::LEN * P::SIZE));
        let v10 = P::load::<V>(self.v.add(v10_offset));
        let v11 = P::load::<V>(self.v.add(v10_offset + V::LEN * P::SIZE));

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u00 = u00.vdivf(self.max_value);
        let u01 = u01.vdivf(self.max_value);
        let u10 = u10.vdivf(self.max_value);
        let u11 = u11.vdivf(self.max_value);

        let v00 = v00.vdivf(self.max_value);
        let v01 = v01.vdivf(self.max_value);
        let v10 = v10.vdivf(self.max_value);
        let v11 = v11.vdivf(self.max_value);

        I444Block {
            px00: I444Pixel {
                y: y00,
                u: u00,
                v: v00,
            },
            px01: I444Pixel {
                y: y01,
                u: u01,
                v: v01,
            },
            px10: I444Pixel {
                y: y10,
                u: u10,
                v: v10,
            },
            px11: I444Pixel {
                y: y11,
                u: u11,
                v: v11,
            },
        }
    }
}
