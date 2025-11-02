use super::{Yuv420Block, Yuv420Src};
use crate::formats::rgb::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;
use crate::{ColorInfo, ColorSpace, ColorTransfer, ConvertError};

/// YUV 420 to RGB converter source
pub(crate) struct ToRgb<S> {
    yuv420_src: S,

    space: ColorSpace,
    transfer: ColorTransfer,
    xyz_to_rgb: &'static [[f32; 3]; 3],
    full_range: bool,
}

impl<S: Yuv420Src> ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, yuv420_src: S) -> Result<Self, ConvertError> {
        let ColorInfo::YUV(yuv) = color else {
            return Err(ConvertError::InvalidColorInfo);
        };

        Ok(Self {
            yuv420_src,
            space: yuv.space,
            transfer: yuv.transfer,
            xyz_to_rgb: yuv.primaries.xyz_to_rgb_mat(),
            full_range: yuv.full_range,
        })
    }
}

impl<S: Yuv420Src> RgbaSrc for ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let Yuv420Block {
            mut y00,
            mut y01,
            mut y10,
            mut y11,
            mut u,
            mut v,
        } = self.yuv420_src.read::<V>(x, y);

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

            u = u.vsub(v16);
            v = v.vsub(v16);

            u = u.vmul(uv_scale);
            v = v.vmul(uv_scale);
        }

        u = u.vsubf(0.5);
        v = v.vsubf(0.5);

        let [
            [r00, g00, b00],
            [r01, g01, b01],
            [r10, g10, b10],
            [r11, g11, b11],
        ] = self
            .space
            .yx4_uv_to_rgb(self.transfer, self.xyz_to_rgb, y00, y01, y10, y11, u, v);

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
