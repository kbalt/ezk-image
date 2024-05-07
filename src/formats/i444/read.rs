use super::{I444Block, I444Src};
use crate::primitive::PrimitiveInternal;
use crate::vector::Vector;
use crate::{ConvertError, I444Pixel, PixelFormat, PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I444Reader<'a, P: PrimitiveInternal> {
    window: Rect,

    src_width: usize,
    y: *const P,
    u: *const P,
    v: *const P,

    max_value: f32,

    _m: PhantomData<&'a [P]>,
}

impl<'a, P: PrimitiveInternal> I444Reader<'a, P> {
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

        let PixelFormatPlanes::I444 { y, u, v } = src_planes else {
            return Err(ConvertError::InvalidPlanesForPixelFormat(PixelFormat::I444));
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

impl<P: PrimitiveInternal> I444Src for I444Reader<'_, P> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I444Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let px00_offset = (y * self.src_width) + x;
        let px10_offset = ((y + 1) * self.src_width) + x;

        // Load Y pixels
        let y00 = P::load::<V>(self.y.add(px00_offset));
        let y01 = P::load::<V>(self.y.add(px00_offset + V::LEN));
        let y10 = P::load::<V>(self.y.add(px10_offset));
        let y11 = P::load::<V>(self.y.add(px10_offset + V::LEN));

        // Load U pixels
        let u00 = P::load::<V>(self.u.add(px00_offset));
        let u01 = P::load::<V>(self.u.add(px00_offset + V::LEN));
        let u10 = P::load::<V>(self.u.add(px10_offset));
        let u11 = P::load::<V>(self.u.add(px10_offset + V::LEN));

        // Load V pixels
        let v00 = P::load::<V>(self.v.add(px00_offset));
        let v01 = P::load::<V>(self.v.add(px00_offset + V::LEN));
        let v10 = P::load::<V>(self.v.add(px10_offset));
        let v11 = P::load::<V>(self.v.add(px10_offset + V::LEN));

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
