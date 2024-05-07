use crate::formats::{I420Block, I420Src};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, PixelFormat, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct NV12Reader<'a, P: PrimitiveInternal> {
    window: Rect,

    src_width: usize,
    src_y: *const P,
    src_uv: *const P,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> NV12Reader<'a, P> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [P]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Result<Self, ConvertError> {
        if !src_planes.bounds_check(src_width, src_height) {
            return Err(ConvertError::InvalidPlaneSizeForDimensions);
        }

        let PixelFormatPlanes::NV12 { y, uv } = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::NV12));
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
            src_y: y.as_ptr(),
            src_uv: uv.as_ptr(),
            max_value: crate::formats::max_value_for_bits(bits_per_component),
            _m: PhantomData,
        })
    }
}

impl<P: PrimitiveInternal> I420Src for NV12Reader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let uv_offset = ((y / 2) * (self.src_width / 2) + (x / 2)) * 2;

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = P::load::<V>(self.src_y.add(y00_offset));
        let y01 = P::load::<V>(self.src_y.add(y00_offset + V::LEN));
        let y10 = P::load::<V>(self.src_y.add(y10_offset));
        let y11 = P::load::<V>(self.src_y.add(y10_offset + V::LEN));

        // Load U and V
        let uv0 = P::load::<V>(self.src_uv.add(uv_offset));
        let uv1 = P::load::<V>(self.src_uv.add(uv_offset + V::LEN));

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
