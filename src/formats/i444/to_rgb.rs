use super::{I444Block, I444Pixel, I444Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbPixel, RgbSrc};
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;

pub(crate) struct I444ToRgb<S> {
    i444_src: S,

    color: ColorOps,
    full_range: bool,
}

impl<S: I444Src> I444ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, i444_src: S) -> Self {
        Self {
            i444_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: I444Src> RgbSrc for I444ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        let I444Block {
            px00,
            px01,
            px10,
            px11,
        } = self.i444_src.read::<V>(x, y);

        RgbBlock {
            rgb00: convert_yuv_to_rgb(&self.color, self.full_range, px00),
            rgb01: convert_yuv_to_rgb(&self.color, self.full_range, px01),
            rgb10: convert_yuv_to_rgb(&self.color, self.full_range, px10),
            rgb11: convert_yuv_to_rgb(&self.color, self.full_range, px11),
        }
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

impl<S: I444Src> RgbaSrc for I444ToRgb<S> {
    forward_rgb_rgba!();
}
