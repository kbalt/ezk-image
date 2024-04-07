use super::{I420Block, I420Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbSrc};
use crate::vector::Vector;

pub(crate) struct RgbToI420<S> {
    rgb_src: S,
    color: ColorOps,
    full_range: bool,
}

impl<S: RgbSrc> RgbToI420<S> {
    pub(crate) fn new(color: &ColorInfo, rgb_src: S) -> Self {
        Self {
            rgb_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: RgbSrc> I420Src for RgbToI420<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let color = V::color_ops(&self.color);

        let RgbBlock {
            rgb00,
            rgb01,
            rgb10,
            rgb11,
        } = self.rgb_src.read(x, y);

        let ([y00, y01, y10, y11], u, v) = self.color.space.rgbx4_to_yx4_uv(
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

        I420Block {
            y00,
            y01,
            y10,
            y11,
            u,
            v,
        }
    }
}
