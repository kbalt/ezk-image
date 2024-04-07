use super::rgb::{RgbBlock, RgbSrc};
use super::rgba::{RgbaBlock, RgbaSrc};
use crate::color::primaries::{rgb_to_xyz, xyz_to_rgb};
use crate::color::{ColorInfo, ColorOps};
use crate::vector::Vector;

pub(crate) struct TransferAndPrimariesConvert<S> {
    src_color: ColorOps,
    dst_color: ColorOps,

    src: S,
}

pub(crate) fn need_transfer_and_primaries_convert(
    src_color: &ColorInfo,
    dst_color: &ColorInfo,
) -> bool {
    src_color.transfer != dst_color.transfer || src_color.primaries != dst_color.primaries
}

impl<S> TransferAndPrimariesConvert<S> {
    pub(crate) fn new(src_color: &ColorInfo, dst_color: &ColorInfo, src: S) -> Self {
        Self {
            src_color: ColorOps::from_info(src_color),
            dst_color: ColorOps::from_info(dst_color),
            src,
        }
    }

    #[rustfmt::skip]
    #[inline(always)]
    unsafe fn convert_primaries<V>(&mut self, i: &mut [&mut V; 12])
    where
        V: Vector,
    {
        let fw = self.src_color.rgb_to_xyz;
        let bw = self.dst_color.xyz_to_rgb;

        let mut iter = i.chunks_exact_mut(3);

        while let Some([r, g, b]) = iter.next() {
            let [x, y, z] = rgb_to_xyz(fw, **r, **g, **b);

            let [r_, g_, b_] =  xyz_to_rgb(bw, x, y, z);

            **r = r_;
            **g = g_;
            **b = b_;
        }
    }

    #[inline(always)]
    unsafe fn convert_rgb<V>(&mut self, mut i: [&mut V; 12])
    where
        V: Vector,
    {
        V::color_ops(&self.src_color)
            .transfer
            .scaled_to_linear12(&mut i);

        self.convert_primaries(&mut i);

        V::color_ops(&self.dst_color)
            .transfer
            .linear_to_scaled12(&mut i);
    }
}

impl<S: RgbSrc> RgbSrc for TransferAndPrimariesConvert<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbBlock<V> {
        let mut block = self.src.read(x, y);

        let i = [
            &mut block.rgb00.r,
            &mut block.rgb00.g,
            &mut block.rgb00.b,
            &mut block.rgb01.r,
            &mut block.rgb01.g,
            &mut block.rgb01.b,
            &mut block.rgb10.r,
            &mut block.rgb10.g,
            &mut block.rgb10.b,
            &mut block.rgb11.r,
            &mut block.rgb11.g,
            &mut block.rgb11.b,
        ];

        self.convert_rgb(i);

        block
    }
}

impl<S: RgbaSrc> RgbaSrc for TransferAndPrimariesConvert<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let mut block = self.src.read(x, y);

        let i = [
            &mut block.rgba00.r,
            &mut block.rgba00.g,
            &mut block.rgba00.b,
            &mut block.rgba01.r,
            &mut block.rgba01.g,
            &mut block.rgba01.b,
            &mut block.rgba10.r,
            &mut block.rgba10.g,
            &mut block.rgba10.b,
            &mut block.rgba11.r,
            &mut block.rgba11.g,
            &mut block.rgba11.b,
        ];

        self.convert_rgb(i);

        block
    }
}
