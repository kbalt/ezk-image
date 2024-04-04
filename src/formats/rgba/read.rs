use super::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::bits::BitsInternal;
use crate::formats::reader::read;
use crate::formats::reader::ImageReader;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct RgbaReader<const REVERSE: bool, B, Vis>
where
    B: BitsInternal,
    Vis: RgbaBlockVisitor,
{
    src_width: usize,
    src: *const B::Primitive,

    max_value: f32,

    visitor: Vis,

    _b: PhantomData<B>,
}

impl<const REVERSE: bool, B, Vis> RgbaReader<REVERSE, B, Vis>
where
    B: BitsInternal,
    Vis: RgbaBlockVisitor,
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

        let PixelFormatPlanes::RGBA(src) = src_planes else {
            panic!("Invalid PixelFormatPlanes for read_rgba");
        };

        read(
            src_width,
            src_height,
            window,
            Self {
                src_width,
                src: src.as_ptr(),
                max_value: crate::max_value_for_bits(bits_per_component),
                visitor,
                _b: PhantomData,
            },
        )
    }
}

impl<const REVERSE: bool, B, Vis> ImageReader for RgbaReader<REVERSE, B, Vis>
where
    B: BitsInternal,
    Vis: RgbaBlockVisitor,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, window: Rect, x: usize, y: usize) {
        let rgba00offset = (y * self.src_width) + x;
        let rgba10offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<REVERSE, B, V, Vis>(
            self.src,
            rgba00offset,
            rgba10offset,
            self.max_value,
            &mut self.visitor,
            x - window.x,
            y - window.y,
        );
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn load_and_visit_block<const REVERSE: bool, B, V, Vis>(
    src_ptr: *const B::Primitive,
    rgba00offset: usize,
    rgba10offset: usize,
    max_value: f32,
    visitor: &mut Vis,
    x: usize,
    y: usize,
) where
    B: BitsInternal,
    V: Vector,
    Vis: RgbaBlockVisitor,
{
    let [[r00, g00, b00, a00], [r01, g01, b01, a01]] =
        B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba00offset * 4));
    let [[r10, g10, b10, a10], [r11, g11, b11, a11]] =
        B::load_4x_interleaved_2x::<V>(src_ptr.add(rgba10offset * 4));

    let r00 = r00.vdivf(max_value);
    let g00 = g00.vdivf(max_value);
    let b00 = b00.vdivf(max_value);
    let a00 = a00.vdivf(max_value);

    let r01 = r01.vdivf(max_value);
    let g01 = g01.vdivf(max_value);
    let b01 = b01.vdivf(max_value);
    let a01 = a01.vdivf(max_value);

    let r10 = r10.vdivf(max_value);
    let g10 = g10.vdivf(max_value);
    let b10 = b10.vdivf(max_value);
    let a10 = a10.vdivf(max_value);

    let r11 = r11.vdivf(max_value);
    let g11 = g11.vdivf(max_value);
    let b11 = b11.vdivf(max_value);
    let a11 = a11.vdivf(max_value);

    let px00 = RgbaPixel::from_loaded::<REVERSE>(r00, g00, b00, a00);
    let px01 = RgbaPixel::from_loaded::<REVERSE>(r01, g01, b01, a01);
    let px10 = RgbaPixel::from_loaded::<REVERSE>(r10, g10, b10, a10);
    let px11 = RgbaPixel::from_loaded::<REVERSE>(r11, g11, b11, a11);

    let block = RgbaBlock {
        rgba00: px00,
        rgba01: px01,
        rgba10: px10,
        rgba11: px11,
    };

    visitor.visit(x, y, block);
}
