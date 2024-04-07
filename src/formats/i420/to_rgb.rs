use super::{I420Block, I420Src};
use crate::color::{ColorInfo, ColorOps};
use crate::formats::rgb::{RgbBlock, RgbPixel, RgbSrc};
use crate::formats::rgba::{RgbaBlock, RgbaPixel, RgbaSrc};
use crate::vector::Vector;

pub(crate) struct I420ToRgb<S> {
    i420_src: S,

    color: ColorOps,
    full_range: bool,
}

impl<S: I420Src> I420ToRgb<S> {
    pub(crate) fn new(color: &ColorInfo, i420_src: S) -> Self {
        Self {
            i420_src,
            color: ColorOps::from_info(color),
            full_range: color.full_range,
        }
    }
}

impl<S: I420Src> RgbSrc for I420ToRgb<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        let color = V::color_ops(&self.color);

        let I420Block {
            mut y00,
            mut y01,
            mut y10,
            mut y11,
            mut u,
            mut v,
        } = self.i420_src.read::<V>(x, y);

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

        let [[r00, g00, b00], [r01, g01, b01], [r10, g10, b10], [r11, g11, b11]] =
            self.color.space.yx4_uv_to_rgb(
                color.transfer,
                self.color.xyz_to_rgb,
                y00,
                y01,
                y10,
                y11,
                u,
                v,
            );

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

impl<S: I420Src> RgbaSrc for I420ToRgb<S> {
    forward_rgb_rgba!();
}
