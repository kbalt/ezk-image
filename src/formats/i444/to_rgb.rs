use super::{I444Block, I444Pixel, I444Src};
use crate::color::ColorInfo;
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;
use crate::{ColorSpace, ColorTransfer, ConvertError};

pub(crate) struct I444ToRgb<S> {
    i444_src: S,

    space: ColorSpace,
    transfer: ColorTransfer,
    xyz_to_rgb: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: I444Src> I444ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, i444_src: S) -> Result<Self, ConvertError> {
        let ColorInfo::YUV(yuv) = color else {
            return Err(ConvertError::InvalidColorInfo);
        };

        Ok(Self {
            i444_src,
            space: yuv.space,
            transfer: yuv.transfer,
            xyz_to_rgb: yuv.primaries.xyz_to_rgb_mat(),
            full_range: yuv.full_range,
        })
    }

    #[inline(always)]
    unsafe fn convert_yuv_to_rgb<V: Vector>(&self, px: I444Pixel<V>) -> RgbaPixel<V> {
        let I444Pixel {
            mut y,
            mut u,
            mut v,
        } = px;

        // If the YUV components don't use the full range of u8 scale them to the full range
        // Y  16..=235 -> 0..255 Y  = (1.164 * (Y  - 16))
        // UV 16..=240 -> 0..255 UV = (1.138 * (UV - 16))
        if !self.full_range {
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

        let (r, g, b) = self
            .space
            .yuv_to_rgb(self.transfer, self.xyz_to_rgb, y, u, v);

        RgbaPixel {
            r,
            g,
            b,
            a: V::splat(1.0),
        }
    }
}

impl<S: I444Src> RgbaSrc for I444ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let I444Block {
            px00,
            px01,
            px10,
            px11,
        } = self.i444_src.read::<V>(x, y);

        RgbaBlock {
            px00: self.convert_yuv_to_rgb(px00),
            px01: self.convert_yuv_to_rgb(px01),
            px10: self.convert_yuv_to_rgb(px10),
            px11: self.convert_yuv_to_rgb(px11),
        }
    }
}
