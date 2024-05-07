use super::{I420Block, I420Src};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I420Reader<'a, P: PrimitiveInternal> {
    window: Rect,

    src_width: usize,
    y: *const P,
    u: *const P,
    v: *const P,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> I420Reader<'a, P> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [P]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Result<Self, ConvertError> {
        if src_planes.bounds_check(src_width, src_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::I420 { y, u, v } = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::I420));
        };

        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: src_width,
            height: src_height,
        });

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Ok(Self {
            window,
            src_width,
            y: y.as_ptr(),
            u: u.as_ptr(),
            v: v.as_ptr(),
            max_value: crate::formats::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        })
    }
}

impl<'a, P: PrimitiveInternal> I420Src for I420Reader<'a, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let uv_offset = (y / 2) * (self.src_width / 2) + (x / 2);

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(y00_offset));
        let y01 = P::load::<V>(self.y.add(y00_offset + V::LEN));
        let y10 = P::load::<V>(self.y.add(y10_offset));
        let y11 = P::load::<V>(self.y.add(y10_offset + V::LEN));

        // Load U and V
        let u = P::load::<V>(self.u.add(uv_offset));
        let v = P::load::<V>(self.v.add(uv_offset));

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
