use super::{I422Block, I422Src};
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;
use crate::{ColorInfo, ColorSpace, ColorTransfer, ConvertError};

pub(crate) struct RgbToI422<S> {
    rgba_src: S,
    space: ColorSpace,
    transfer: ColorTransfer,
    rgb_to_xyz: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: RgbaSrc> RgbToI422<S> {
    pub(crate) fn new(color: &ColorInfo, rgba_src: S) -> Result<Self, ConvertError> {
        let ColorInfo::YUV(yuv) = color else {
            return Err(ConvertError::MismatchedImageSize);
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
    unsafe fn convert_rgb_to_yuv<V: Vector>(
        &self,
        px0: RgbaPixel<V>,
        px1: RgbaPixel<V>,
    ) -> ([V; 2], V, V) {
        let (mut y0, u0, v0) =
            self.space
                .rgb_to_yuv(self.transfer, self.rgb_to_xyz, px0.r, px0.g, px0.b);

        let (mut y1, u1, v1) =
            self.space
                .rgb_to_yuv(self.transfer, self.rgb_to_xyz, px1.r, px1.g, px1.b);

        let (u0, u1) = u0.unzip(u1);
        let (v0, v1) = v0.unzip(v1);

        let mut u = u0.vadd(u1).vdivf(2.0);
        let mut v = v0.vadd(v1).vdivf(2.0);

        // U & V scales from -0.5..=0.5, so bring that up into 0..=1
        u = u.vaddf(0.5);
        v = v.vaddf(0.5);

        if !self.full_range {
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

        let ([y00, y01], u0, v0) = self.convert_rgb_to_yuv(px00, px01);
        let ([y10, y11], u1, v1) = self.convert_rgb_to_yuv(px10, px11);

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
