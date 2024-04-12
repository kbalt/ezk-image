use super::{I422Block, I422Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;

pub(crate) struct RgbToI422<S> {
    rgba_src: S,
    color: ColorOps,
    full_range: bool,
}

impl<S: RgbaSrc> RgbToI422<S> {
    pub(crate) fn new(color: &ColorInfo, rgba_src: S) -> Self {
        Self {
            rgba_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: RgbaSrc> I422Src for RgbToI422<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I422Block<V> {
        let RgbaBlock {
            px00,
            px01,
            px10,
            px11,
        } = self.rgba_src.read(x, y);

        let ([y00, y01], u0, v0) = convert_rgb_to_yuv(&self.color, self.full_range, px00, px01);
        let ([y10, y11], u1, v1) = convert_rgb_to_yuv(&self.color, self.full_range, px10, px11);

        I422Block {
            y00,
            y01,
            y10,
            y11,
            u0,
            u1,
            v0,
            v1,
        }
    }
}

#[inline(always)]
unsafe fn convert_rgb_to_yuv<V: Vector>(
    color: &ColorOps,
    full_range: bool,
    px0: RgbaPixel<V>,
    px1: RgbaPixel<V>,
) -> ([V; 2], V, V) {
    let (mut y0, u0, v0) =
        color
            .space
            .rgb_to_yuv(color.transfer, color.rgb_to_xyz, px0.r, px0.g, px0.b);

    let (mut y1, u1, v1) =
        color
            .space
            .rgb_to_yuv(color.transfer, color.rgb_to_xyz, px1.r, px1.g, px1.b);

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