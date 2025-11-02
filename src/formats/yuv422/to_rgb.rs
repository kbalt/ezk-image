use super::Yuv422Block;
use crate::formats::rgb::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::formats::yuv422::Yuv422Src;
use crate::vector::Vector;
use crate::{ColorInfo, ColorSpace, ColorTransfer, ConvertError};

/// YUV 422 to RGB converter source
pub(crate) struct ToRgb<S> {
    yuv422_src: S,

    space: ColorSpace,
    transfer: ColorTransfer,
    xyz_to_rgb: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: Yuv422Src> ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, yuv422_src: S) -> Result<Self, ConvertError> {
        let ColorInfo::YUV(yuv) = color else {
            return Err(ConvertError::InvalidColorInfo);
        };

        Ok(Self {
            yuv422_src,
            space: yuv.space,
            transfer: yuv.transfer,
            xyz_to_rgb: yuv.primaries.xyz_to_rgb_mat(),
            full_range: yuv.full_range,
        })
    }
}

impl<S: Yuv422Src> RgbaSrc for ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let Yuv422Block {
            mut y00,
            mut y01,
            mut y10,
            mut y11,
            mut u0,
            mut u1,
            mut v0,
            mut v1,
        } = self.yuv422_src.read::<V>(x, y);

        // If the YUV components don't use the full range of u8 scale them to the full range
        // Y  16..=235 -> 0..255 Y  = (1.164 * (Y  - 16))
        // UV 16..=240 -> 0..255 UV = (1.138 * (UV - 16))
        if !self.full_range {
            let v16 = V::splat(16.0 / 255.0);
            let y_scale = V::splat(255.0 / 219.0);
            let uv_scale = V::splat(255.0 / 224.0);

            y00 = y00.vsub(v16);
            y01 = y01.vsub(v16);
            y10 = y10.vsub(v16);
            y11 = y11.vsub(v16);

            y00 = y00.vmul(y_scale);
            y01 = y01.vmul(y_scale);
            y10 = y10.vmul(y_scale);
            y11 = y11.vmul(y_scale);

            u0 = u0.vsub(v16);
            u1 = u1.vsub(v16);

            v0 = v0.vsub(v16);
            v1 = v1.vsub(v16);

            u0 = u0.vmul(uv_scale);
            u1 = u1.vmul(uv_scale);

            v0 = v0.vmul(uv_scale);
            v1 = v1.vmul(uv_scale);
        }

        u0 = u0.vsubf(0.5);
        u1 = u1.vsubf(0.5);
        v0 = v0.vsubf(0.5);
        v1 = v1.vsubf(0.5);

        let (u0_left, u0_right) = u0.zip(u0);
        let (v0_left, v0_right) = v0.zip(v0);

        let (r00, g00, b00) =
            self.space
                .yuv_to_rgb(self.transfer, self.xyz_to_rgb, y00, u0_left, v0_left);
        let (r01, g01, b01) =
            self.space
                .yuv_to_rgb(self.transfer, self.xyz_to_rgb, y01, u0_right, v0_right);

        let (u1_left, u1_right) = u1.zip(u1);
        let (v1_left, v1_right) = v1.zip(v1);

        let (r10, g10, b10) =
            self.space
                .yuv_to_rgb(self.transfer, self.xyz_to_rgb, y10, u1_left, v1_left);
        let (r11, g11, b11) =
            self.space
                .yuv_to_rgb(self.transfer, self.xyz_to_rgb, y11, u1_right, v1_right);

        RgbaBlock {
            px00: RgbaPixel {
                r: r00,
                g: g00,
                b: b00,
                a: V::splat(1.0),
            },
            px01: RgbaPixel {
                r: r01,
                g: g01,
                b: b01,
                a: V::splat(1.0),
            },
            px10: RgbaPixel {
                r: r10,
                g: g10,
                b: b10,
                a: V::splat(1.0),
            },
            px11: RgbaPixel {
                r: r11,
                g: g11,
                b: b11,
                a: V::splat(1.0),
            },
        }
    }
}
