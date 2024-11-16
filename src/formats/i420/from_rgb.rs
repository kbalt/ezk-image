use super::{I420Block, I420Src};
use crate::color::ColorInfo;
use crate::formats::rgba::{RgbaBlock, RgbaSrc};
use crate::vector::Vector;
use crate::{ColorSpace, ColorTransfer, ConvertError};

pub(crate) struct RgbToI420<S> {
    rgba_src: S,
    space: ColorSpace,
    transfer: ColorTransfer,
    rgb_to_xyz: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: RgbaSrc> RgbToI420<S> {
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
}

impl<S: RgbaSrc> I420Src for RgbToI420<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let RgbaBlock {
            px00,
            px01,
            px10,
            px11,
        } = self.rgba_src.read::<V>(x, y);

        let ([y00, y01, y10, y11], u, v) = self.space.rgbx4_to_yx4_uv(
            self.transfer,
            self.rgb_to_xyz,
            [px00.r, px01.r, px10.r, px11.r],
            [px00.g, px01.g, px10.g, px11.g],
            [px00.b, px01.b, px10.b, px11.b],
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
