use super::{I422Block, I422Visitor};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::vector::Vector;

pub(crate) struct RgbToI422Visitor<Vis> {
    visitor: Vis,
    color: ColorOps,
    full_range: bool,
}

impl<Vis> RgbToI422Visitor<Vis>
where
    Vis: I422Visitor,
{
    pub(crate) fn new(color: &ColorInfo, visitor: Vis) -> Self {
        Self {
            visitor,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<Vis: I422Visitor> RgbaBlockVisitor for RgbToI422Visitor<Vis> {
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

impl<Vis: I422Visitor> RgbBlockVisitor for RgbToI422Visitor<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        let RgbBlock {
            rgb00,
            rgb01,
            rgb10,
            rgb11,
        } = block;

        let ([y00, y01], u0, v0) = convert_rgb_to_yuv(&self.color, self.full_range, rgb00, rgb01);
        let ([y10, y11], u1, v1) = convert_rgb_to_yuv(&self.color, self.full_range, rgb10, rgb11);

        self.visitor.visit(
            x,
            y,
            I422Block {
                y00,
                y01,
                y10,
                y11,
                u0,
                u1,
                v0,
                v1,
            },
        );
    }
}

#[inline(always)]
unsafe fn convert_rgb_to_yuv<V: Vector>(
    color: &ColorOps,
    full_range: bool,
    px0: RgbPixel<V>,
    px1: RgbPixel<V>,
) -> ([V; 2], V, V) {
    let color_ops = V::color_ops(color);

    let (mut y0, u0, v0) =
        color
            .space
            .rgb_to_yuv(color_ops.transfer, color.rgb_to_xyz, px0.r, px0.g, px0.b);

    let (mut y1, u1, v1) =
        color
            .space
            .rgb_to_yuv(color_ops.transfer, color.rgb_to_xyz, px1.r, px1.g, px1.b);

    let mut u = u0.vadd(u1).vdivf(2.0);
    let mut v = v0.vadd(v1).vdivf(2.0);

    // U & V scales from -0.5..=0.5, so bring that up into 0..=1
    u = u.vaddf(0.5);
    v = v.vaddf(0.5);

    if !full_range {
        let v16 = V::splat(16.0 / 255.0);

        let y_scale = V::splat(219.0 / 255.0);
        let uv_scale = V::splat(224.0 / 255.0);

        y0 = v16.vadd(y0.vmul(y_scale));
        y1 = v16.vadd(y1.vmul(y_scale));

        u = v16.vadd(u.vmul(uv_scale));
        v = v16.vadd(v.vmul(uv_scale));
    }

    ([y0, y1], u, v)
}
