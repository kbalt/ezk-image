use super::{I420Block, I420Visitor};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::formats::rgba::{RgbaBlock, RgbaBlockVisitor, RgbaPixel};
use crate::vector::Vector;

pub(crate) struct RgbToI420Visitor<Vis> {
    visitor: Vis,
    color: ColorOps,
    full_range: bool,
}

impl<Vis> RgbToI420Visitor<Vis>
where
    Vis: I420Visitor,
{
    pub(crate) fn new(color: &ColorInfo, visitor: Vis) -> Self {
        Self {
            visitor,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<Vis: I420Visitor> RgbaBlockVisitor for RgbToI420Visitor<Vis> {
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

impl<Vis: I420Visitor> RgbBlockVisitor for RgbToI420Visitor<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: RgbBlock<V>) {
        let color = V::color_ops(&self.color);

        let RgbBlock {
            rgb00,
            rgb01,
            rgb10,
            rgb11,
        } = block;

        let ([y00, y01, y10, y11], u, v) = color.space.rgbx4_to_yx4_uv(
            color.transfer,
            self.color.rgb_to_xyz,
            [rgb00.r, rgb01.r, rgb10.r, rgb11.r],
            [rgb00.g, rgb01.g, rgb10.g, rgb11.g],
            [rgb00.b, rgb01.b, rgb10.b, rgb11.b],
        );

        // U & V scales from -0.5..=0.5, so bring that up into 0..=1
        let u = u.vaddf(0.5);
        let v = v.vaddf(0.5);

        let (y00, y01, y10, y11, u, v) = if self.full_range {
            (y00, y01, y10, y11, u, v)
        } else {
            let v16 = V::splat(16.0 / 255.0);

            let y_scale = V::splat(219.0 / 255.0);
            let uv_scale = V::splat(224.0 / 255.0);

            let y00 = v16.vadd(y00.vmul(y_scale));
            let y01 = v16.vadd(y01.vmul(y_scale));
            let y10 = v16.vadd(y10.vmul(y_scale));
            let y11 = v16.vadd(y11.vmul(y_scale));

            let u = v16.vadd(u.vmul(uv_scale));
            let v = v16.vadd(v.vmul(uv_scale));

            (y00, y01, y10, y11, u, v)
        };

        self.visitor.visit(
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
}
