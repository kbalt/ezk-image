use super::{I444Block, I444Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::i444::I444Pixel;
use crate::formats::rgba::{RgbaPixel, RgbaSrc};
use crate::vector::Vector;

pub(crate) struct RgbToI444<S> {
    rgba_src: S,
    color: ColorOps,
    full_range: bool,
}

impl<S: RgbaSrc> RgbToI444<S> {
    pub(crate) fn new(color: &ColorInfo, rgba_src: S) -> Self {
        Self {
            rgba_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: RgbaSrc> I444Src for RgbToI444<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I444Block<V> {
        let block = self.rgba_src.read(x, y);

        I444Block {
            px00: convert_rgb_to_yuv(&self.color, self.full_range, block.px00),
            px01: convert_rgb_to_yuv(&self.color, self.full_range, block.px01),
            px10: convert_rgb_to_yuv(&self.color, self.full_range, block.px10),
            px11: convert_rgb_to_yuv(&self.color, self.full_range, block.px11),
        }
    }
}

#[inline(always)]
pub(crate) unsafe fn convert_rgb_to_yuv<V: Vector>(
    color: &ColorOps,
    full_range: bool,
    px: RgbaPixel<V>,
) -> I444Pixel<V> {
    let (mut y, mut u, mut v) =
        color
            .space
            .rgb_to_yuv(color.transfer, color.rgb_to_xyz, px.r, px.g, px.b);

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
