#![allow(clippy::too_many_arguments)]

use std::marker::PhantomData;

use super::{I444Block, I444Pixel, I444Visitor};
use crate::bits::BitsInternal;
use crate::formats::reader::{read, ImageReader};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};

pub(crate) struct I444Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I444Visitor,
{
    src_width: usize,
    src_y: *const B::Primitive,
    src_u: *const B::Primitive,
    src_v: *const B::Primitive,

    max_value: f32,

    visitor: Vis,

    _b: PhantomData<B>,
}

impl<B, Vis> I444Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I444Visitor,
{
    pub(crate) fn read(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&[B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
        visitor: Vis,
    ) {
        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::I444 { y, u, v } = src_planes else {
            panic!("Invalid PixelFormatPlanes for read_i444");
        };

        read(
            src_width,
            src_height,
            window,
            Self {
                src_width,
                src_y: y.as_ptr(),
                src_u: u.as_ptr(),
                src_v: v.as_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                visitor,
                _b: PhantomData,
            },
        )
    }
}

impl<B, Vis> ImageReader for I444Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I444Visitor,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, window: Rect, x: usize, y: usize) {
        let px00_offset = (y * self.src_width) + x;
        let px10_offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<V, B, Vis>(
            &mut self.visitor,
            x - window.x,
            y - window.y,
            self.src_y,
            self.src_u,
            self.src_v,
            px00_offset,
            px10_offset,
            self.max_value,
        );
    }
}

#[inline(always)]
unsafe fn load_and_visit_block<V, B, Vis>(
    visitor: &mut Vis,
    x: usize,
    y: usize,
    y_ptr: *const B::Primitive,
    u_ptr: *const B::Primitive,
    v_ptr: *const B::Primitive,
    px00_offset: usize,
    px10_offset: usize,
    max_value: f32,
) where
    V: Vector,
    Vis: I444Visitor,
    B: BitsInternal,
{
    // Load Y pixels
    let y00 = B::load::<V>(y_ptr.add(px00_offset));
    let y01 = B::load::<V>(y_ptr.add(px00_offset + V::LEN));
    let y10 = B::load::<V>(y_ptr.add(px10_offset));
    let y11 = B::load::<V>(y_ptr.add(px10_offset + V::LEN));

    // Load U pixels
    let u00 = B::load::<V>(u_ptr.add(px00_offset));
    let u01 = B::load::<V>(u_ptr.add(px00_offset + V::LEN));
    let u10 = B::load::<V>(u_ptr.add(px10_offset));
    let u11 = B::load::<V>(u_ptr.add(px10_offset + V::LEN));

    // Load V pixels
    let v00 = B::load::<V>(v_ptr.add(px00_offset));
    let v01 = B::load::<V>(v_ptr.add(px00_offset + V::LEN));
    let v10 = B::load::<V>(v_ptr.add(px10_offset));
    let v11 = B::load::<V>(v_ptr.add(px10_offset + V::LEN));

    // Convert to analog 0..=1.0
    let y00 = y00.vdivf(max_value);
    let y01 = y01.vdivf(max_value);
    let y10 = y10.vdivf(max_value);
    let y11 = y11.vdivf(max_value);

    let u00 = u00.vdivf(max_value);
    let u01 = u01.vdivf(max_value);
    let u10 = u10.vdivf(max_value);
    let u11 = u11.vdivf(max_value);

    let v00 = v00.vdivf(max_value);
    let v01 = v01.vdivf(max_value);
    let v10 = v10.vdivf(max_value);
    let v11 = v11.vdivf(max_value);

    visitor.visit(
        x,
        y,
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
        },
    );
}
