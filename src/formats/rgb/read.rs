use super::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::bits::BitsInternal;
use crate::formats::reader::read;
use crate::formats::reader::ImageReader;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct RgbReader<const REVERSE: bool, B, Vis>
where
    B: BitsInternal,
    Vis: RgbBlockVisitor,
{
    src_width: usize,
    src: *const B::Primitive,

    max_value: f32,

    visitor: Vis,

    _b: PhantomData<B>,
}

impl<const REVERSE: bool, B, Vis> RgbReader<REVERSE, B, Vis>
where
    B: BitsInternal,
    Vis: RgbBlockVisitor,
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

        let PixelFormatPlanes::RGB(src) = src_planes else {
            panic!("Invalid PixelFormatPlanes for read_rgb");
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

impl<const REVERSE: bool, B, Vis> ImageReader for RgbReader<REVERSE, B, Vis>
where
    B: BitsInternal,
    Vis: RgbBlockVisitor,
{
    #[inline(always)]
    unsafe fn read_at<V: Vector>(&mut self, window: Rect, x: usize, y: usize) {
        let rgb00offset = (y * self.src_width) + x;
        let rgb10offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<REVERSE, B, V, Vis>(
            self.src,
            rgb00offset,
            rgb10offset,
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
    rgb00offset: usize,
    rgb10offset: usize,
    max_value: f32,
    visitor: &mut Vis,
    x: usize,
    y: usize,
) where
    B: BitsInternal,
    V: Vector,
    Vis: RgbBlockVisitor,
{
    let [[r00, g00, b00], [r01, g01, b01]] =
        B::load_3x_interleaved_2x::<V>(src_ptr.add(rgb00offset * 3));
    let [[r10, g10, b10], [r11, g11, b11]] =
        B::load_3x_interleaved_2x::<V>(src_ptr.add(rgb10offset * 3));

    let r00 = r00.vdivf(max_value);
    let g00 = g00.vdivf(max_value);
    let b00 = b00.vdivf(max_value);

    let r01 = r01.vdivf(max_value);
    let g01 = g01.vdivf(max_value);
    let b01 = b01.vdivf(max_value);

    let r10 = r10.vdivf(max_value);
    let g10 = g10.vdivf(max_value);
    let b10 = b10.vdivf(max_value);

    let r11 = r11.vdivf(max_value);
    let g11 = g11.vdivf(max_value);
    let b11 = b11.vdivf(max_value);

    let px00 = RgbPixel::from_loaded::<REVERSE>(r00, g00, b00);
    let px01 = RgbPixel::from_loaded::<REVERSE>(r01, g01, b01);
    let px10 = RgbPixel::from_loaded::<REVERSE>(r10, g10, b10);
    let px11 = RgbPixel::from_loaded::<REVERSE>(r11, g11, b11);

    let block = RgbBlock {
        rgb00: px00,
        rgb01: px01,
        rgb10: px10,
        rgb11: px11,
    };

    visitor.visit(x, y, block);
}
