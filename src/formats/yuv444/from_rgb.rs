use super::{Yuv444Block, Yuv444Src};
use crate::color::ColorInfo;
use crate::formats::rgb::{RgbaPixel, RgbaSrc};
use crate::formats::yuv444::Yuv444Pixel;
use crate::vector::Vector;
use crate::{ColorSpace, ColorTransfer, ConvertError};

/// RGB to YUV444 converter source
pub(crate) struct FromRgb<S> {
    rgba_src: S,
    space: ColorSpace,
    transfer: ColorTransfer,
    rgb_to_xyz: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: RgbaSrc> FromRgb<S> {
    pub(crate) fn new(color: &ColorInfo, rgba_src: S) -> Result<Self, ConvertError> {
        let ColorInfo::YUV(yuv) = color else {
            return Err(ConvertError::InvalidColorInfo);
        };

        Ok(Self {
            rgba_src,
            space: yuv.space,
            transfer: yuv.transfer,
            rgb_to_xyz: yuv.primaries.rgb_to_xyz_mat(),
            full_range: yuv.full_range,
        })
    }

    #[inline(always)]
    unsafe fn convert_rgb_to_yuv<V: Vector>(&self, px: RgbaPixel<V>) -> Yuv444Pixel<V> {
        let (mut y, mut u, mut v) =
            self.space
                .rgb_to_yuv(self.transfer, self.rgb_to_xyz, px.r, px.g, px.b);

        // U & V scales from -0.5..=0.5, so bring that up into 0..=1
        u = u.vaddf(0.5);
        v = v.vaddf(0.5);

        if !self.full_range {
            let v16 = V::splat(16.0 / 255.0);

            let y_scale = V::splat(219.0 / 255.0);
            let uv_scale = V::splat(224.0 / 255.0);

            y = v16.vadd(y.vmul(y_scale));

            u = v16.vadd(u.vmul(uv_scale));
            v = v16.vadd(v.vmul(uv_scale));
        }

        Yuv444Pixel { y, u, v }
    }
}

impl<S: RgbaSrc> Yuv444Src for FromRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> Yuv444Block<V> {
        let block = self.rgba_src.read(x, y);

        Yuv444Block {
            px00: self.convert_rgb_to_yuv(block.px00),
            px01: self.convert_rgb_to_yuv(block.px01),
            px10: self.convert_rgb_to_yuv(block.px10),
            px11: self.convert_rgb_to_yuv(block.px11),
        }
    }
}
