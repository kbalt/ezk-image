use crate::formats::yuv420::{Yuv420Block, Yuv420Src};
use crate::planes::read_planes;
use crate::primitive::Primitive;
use crate::vector::Vector;
use crate::{ConvertError, ImageRef, ImageRefExt};
use std::marker::PhantomData;

pub(crate) struct Read3Plane<'a, P: Primitive> {
    y: &'a [u8],
    u: &'a [u8],
    v: &'a [u8],

    y_stride: usize,
    u_stride: usize,
    v_stride: usize,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: Primitive> Read3Plane<'a, P> {
    pub(crate) fn new(src: &'a dyn ImageRef) -> Result<Self, ConvertError> {
        src.bounds_check()?;

        let [(y, y_stride), (u, u_stride), (v, v_stride)] = read_planes(src.planes())?;

        Ok(Self {
            y,
            u,
            v,
            y_stride,
            u_stride,
            v_stride,
            max_value: crate::formats::max_value_for_bits(src.format().bits_per_component()),
            _m: PhantomData,
        })
    }
}

impl<P: Primitive> Yuv420Src for Read3Plane<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> Yuv420Block<V> {
        let y00_offset = (y * self.y_stride) + x * P::SIZE;
        let y10_offset = ((y + 1) * self.y_stride) + x * P::SIZE;

        let u_offset = (y / 2) * (self.u_stride) + (x / 2) * P::SIZE;
        let v_offset = (y / 2) * (self.v_stride) + (x / 2) * P::SIZE;

        // Load Y pixels
        let y00 = P::load::<V>(&self.y[y00_offset..]);
        let y01 = P::load::<V>(&self.y[y00_offset + V::LEN * P::SIZE..]);
        let y10 = P::load::<V>(&self.y[y10_offset..]);
        let y11 = P::load::<V>(&self.y[y10_offset + V::LEN * P::SIZE..]);

        // Load U and V
        let u = P::load::<V>(&self.u[u_offset..]);
        let v = P::load::<V>(&self.v[v_offset..]);

        // Convert to analog 0..=1.0
        let y00 = y00.vdivf(self.max_value);
        let y01 = y01.vdivf(self.max_value);
        let y10 = y10.vdivf(self.max_value);
        let y11 = y11.vdivf(self.max_value);

        let u = u.vdivf(self.max_value);
        let v = v.vdivf(self.max_value);

        Yuv420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        }
    }
}
