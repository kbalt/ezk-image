#![allow(clippy::too_many_arguments)]

use crate::bits::BitsInternal;
use crate::formats::reader::{read, ImageReader};
use crate::formats::{I420Block, I420Visitor};
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct NV12Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I420Visitor,
{
    src_width: usize,
    src_y: *const B::Primitive,
    src_uv: *const B::Primitive,

    max_value: f32,

    visitor: Vis,

    _b: PhantomData<B>,
}

impl<B, Vis> NV12Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I420Visitor,
{
    #[inline]
    pub fn read(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&[B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
        visitor: Vis,
    ) {
        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::NV12 { y, uv } = src_planes else {
            panic!("Invalid PixelFormatPlanes for read_nv12");
        };

        read(
            src_width,
            src_height,
            window,
            Self {
                src_width,
                src_y: y.as_ptr(),
                src_uv: uv.as_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                visitor,
                _b: PhantomData,
            },
        )
    }
}

impl<B, Vis> ImageReader for NV12Reader<B, Vis>
where
    B: BitsInternal,
    Vis: I420Visitor,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, window: Rect, x: usize, y: usize) {
        let uv_offset = (y / 2) * (self.src_width / 2) + (x / 2);

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<V, B, Vis>(
            &mut self.visitor,
            x - window.x,
            y - window.y,
            self.src_y,
            self.src_uv,
            y00_offset,
            y10_offset,
            uv_offset * 2,
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
    uv_ptr: *const B::Primitive,
    y00_offset: usize,
    y10_offset: usize,
    uv_offset: usize,
    max_value: f32,
) where
    V: Vector,
    Vis: I420Visitor,
    B: BitsInternal,
{
    // Load Y pixels
    let y00 = B::load::<V>(y_ptr.add(y00_offset));
    let y01 = B::load::<V>(y_ptr.add(y00_offset + V::LEN));
    let y10 = B::load::<V>(y_ptr.add(y10_offset));
    let y11 = B::load::<V>(y_ptr.add(y10_offset + V::LEN));

    // Load U and V
    let uv0 = B::load::<V>(uv_ptr.add(uv_offset));
    let uv1 = B::load::<V>(uv_ptr.add(uv_offset + V::LEN));

    let (u, v) = uv0.unzip(uv1);

    // Convert to analog 0..=1.0
    let y00 = y00.vdivf(max_value);
    let y01 = y01.vdivf(max_value);
    let y10 = y10.vdivf(max_value);
    let y11 = y11.vdivf(max_value);

    let u = u.vdivf(max_value);
    let v = v.vdivf(max_value);

    visitor.visit(
        x,
        y,
        I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        },
    );
}
