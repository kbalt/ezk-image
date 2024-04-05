use super::{I444Block, I444Visitor};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::i444::I444Pixel;
use crate::formats::rgb::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::vector::Vector;

pub(crate) struct RgbToI444Visitor<Vis> {
    visitor: Vis,
    color: ColorOps,
    full_range: bool,
}

impl<Vis> RgbToI444Visitor<Vis>
where
    Vis: I444Visitor,
{
    pub(crate) fn new(color: &ColorInfo, visitor: Vis) -> Self {
        Self {
            visitor,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<Vis: I444Visitor> RgbaBlockVisitor for RgbToI444Visitor<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbaBlock<V>) {
        fn conv<V>(px: RgbaPixel<V>) -> RgbPixel<V> {
            RgbPixel {
                r: px.r,
                g: px.g,
                b: px.b,
            }
        }

        RgbBlockVisitor::visit(
            self,
            x,
            y,
            RgbBlock {
                rgb00: conv(block.rgba00),
                rgb01: conv(block.rgba01),
                rgb10: conv(block.rgba10),
                rgb11: conv(block.rgba11),
            },
        )
    }
}

impl<Vis: I444Visitor> RgbBlockVisitor for RgbToI444Visitor<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        self.visitor.visit(
            x,
            y,
            I444Block {
                px00: convert_rgb_to_yuv(&self.color, self.full_range, block.rgb00),
                px01: convert_rgb_to_yuv(&self.color, self.full_range, block.rgb01),
                px10: convert_rgb_to_yuv(&self.color, self.full_range, block.rgb10),
                px11: convert_rgb_to_yuv(&self.color, self.full_range, block.rgb11),
            },
        );
    }
}

#[inline(always)]
pub(crate) unsafe fn convert_rgb_to_yuv<V: Vector>(
    color: &ColorOps,
    full_range: bool,
    px: RgbPixel<V>,
) -> I444Pixel<V> {
    let color_ops = V::color_ops(color);

    let (mut y, mut u, mut v) =
        color
            .space
            .rgb_to_yuv(color_ops.transfer, color.rgb_to_xyz, px.r, px.g, px.b);

    // U & V scales from -0.5..=0.5, so bring that up into 0..=1
    u = u.vaddf(0.5);
    v = v.vaddf(0.5);

    if !full_range {
        let v16 = V::splat(16.0 / 255.0);

        let y_scale = V::splat(219.0 / 255.0);
        let uv_scale = V::splat(224.0 / 255.0);

        y = v16.vadd(y.vmul(y_scale));

        u = v16.vadd(u.vmul(uv_scale));
        v = v16.vadd(v.vmul(uv_scale));
    }

    I444Pixel { y, u, v }
}
