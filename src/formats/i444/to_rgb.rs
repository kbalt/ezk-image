use super::{I444Block, I444Pixel, I444Visitor};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbBlockVisitor, RgbPixel};
use crate::vector::Vector;

pub(crate) struct I444ToRgbVisitor<Vis> {
    visitor: Vis,

    color: ColorOps,
    full_range: bool,
}

impl<Vis> I444ToRgbVisitor<Vis>
where
    Vis: RgbBlockVisitor,
{
    pub(crate) fn new(color: &ColorInfo, visitor: Vis) -> Self {
        Self {
            visitor,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<Vis: RgbBlockVisitor> I444Visitor for I444ToRgbVisitor<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, block: I444Block<V>) {
        let I444Block {
            px00,
            px01,
            px10,
            px11,
        } = block;

        let block = RgbBlock {
            rgb00: convert_yuv_to_rgb(&self.color, self.full_range, px00),
            rgb01: convert_yuv_to_rgb(&self.color, self.full_range, px01),
            rgb10: convert_yuv_to_rgb(&self.color, self.full_range, px10),
            rgb11: convert_yuv_to_rgb(&self.color, self.full_range, px11),
        };

        self.visitor.visit(x, y, block)
    }
}

#[inline(always)]
unsafe fn convert_yuv_to_rgb<V: Vector>(
    color: &ColorOps,
    full_range: bool,
    px: I444Pixel<V>,
) -> RgbPixel<V> {
    let color_ops = V::color_ops(color);

    let I444Pixel {
        mut y,
        mut u,
        mut v,
    } = px;

    // If the YUV components don't use the full range of u8 scale them to the full range
    // Y  16..=235 -> 0..255 Y  = (1.164 * (Y  - 16))
    // UV 16..=240 -> 0..255 UV = (1.138 * (UV - 16))
    if !full_range {
        let v16 = V::splat(16.0 / 255.0);
        let y_scale = V::splat(255.0 / 219.0);
        let uv_scale = V::splat(255.0 / 224.0);

        y = y.vsub(v16);
        y = y.vmul(y_scale);

        u = u.vsub(v16);
        v = v.vsub(v16);

        u = u.vmul(uv_scale);
        v = v.vmul(uv_scale);
    }

    u = u.vsubf(0.5);
    v = v.vsubf(0.5);

    let (r, g, b) = color
        .space
        .yuv_to_rgb(color_ops.transfer, color.xyz_to_rgb, y, u, v);

    RgbPixel { r, g, b }
}
