use super::{I422Block, I422Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbPixel, RgbSrc};
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;

pub(crate) struct I422ToRgb<S> {
    i422_src: S,

    color: ColorOps,
    full_range: bool,
}

impl<S: I422Src> I422ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, i422_src: S) -> Self {
        Self {
            i422_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: I422Src> RgbSrc for I422ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        let color = V::color_ops(&self.color);

        let I422Block {
            mut y00,
            mut y01,
            mut y10,
            mut y11,
            mut u0,
            mut u1,
            mut v0,
            mut v1,
        } = self.i422_src.read::<V>(x, y);

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

            v1 = v1.vsub(v16);
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

        let (r00, g00, b00) =
            self.color
                .space
                .yuv_to_rgb(color.transfer, self.color.xyz_to_rgb, y00, u0, v0);
        let (r01, g01, b01) =
            self.color
                .space
                .yuv_to_rgb(color.transfer, self.color.xyz_to_rgb, y01, u0, v0);

        let (r10, g10, b10) =
            self.color
                .space
                .yuv_to_rgb(color.transfer, self.color.xyz_to_rgb, y10, u1, v1);
        let (r11, g11, b11) =
            self.color
                .space
                .yuv_to_rgb(color.transfer, self.color.xyz_to_rgb, y11, u1, v1);

        RgbBlock {
            rgb00: RgbPixel {
                r: r00,
                g: g00,
                b: b00,
            },
            rgb01: RgbPixel {
                r: r01,
                g: g01,
                b: b01,
            },
            rgb10: RgbPixel {
                r: r10,
                g: g10,
                b: b10,
            },
            rgb11: RgbPixel {
                r: r11,
                g: g11,
                b: b11,
            },
        }
    }
}

impl<S: I422Src> RgbaSrc for I422ToRgb<S> {
    forward_rgb_rgba!();
}
