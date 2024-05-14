use super::rgba::{RgbaBlock, RgbaSrc};
use crate::color::primaries::{rgb_to_xyz, xyz_to_rgb};
use crate::color::ColorInfo;
use crate::vector::Vector;
use crate::ColorTransfer;

pub(crate) struct TransferAndPrimariesConvert<S> {
    rgb_to_xyz: &'static [[f32; 3]; 3],
    xyz_to_rgb: &'static [[f32; 3]; 3],

    src_transfer: ColorTransfer,
    dst_transfer: ColorTransfer,

    src: S,
}

pub(crate) fn need_transfer_and_primaries_convert(
    src_color: &ColorInfo,
    dst_color: &ColorInfo,
) -> bool {
    src_color.transfer() != dst_color.transfer() || src_color.primaries() != dst_color.primaries()
}

impl<S> TransferAndPrimariesConvert<S> {
    pub(crate) fn new(src_color: &ColorInfo, dst_color: &ColorInfo, src: S) -> Self {
        Self {
            rgb_to_xyz: src_color.primaries().rgb_to_xyz_mat(),
            xyz_to_rgb: dst_color.primaries().xyz_to_rgb_mat(),
            src_transfer: src_color.transfer(),
            dst_transfer: dst_color.transfer(),
            src,
        }
    }
}

impl<S: RgbaSrc> RgbaSrc for TransferAndPrimariesConvert<S> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> RgbaBlock<V> {
        let mut block = self.src.read(x, y);

        let mut i = [
            &mut block.px00.r,
            &mut block.px00.g,
            &mut block.px00.b,
            &mut block.px01.r,
            &mut block.px01.g,
            &mut block.px01.b,
            &mut block.px10.r,
            &mut block.px10.g,
            &mut block.px10.b,
            &mut block.px11.r,
            &mut block.px11.g,
            &mut block.px11.b,
        ];

        self.src_transfer.scaled_to_linear_v(&mut i);

        let mut iter = i.chunks_exact_mut(3);

        while let Some([r, g, b]) = iter.next() {
            let [x, y, z] = rgb_to_xyz(self.rgb_to_xyz, **r, **g, **b);

            let [r_, g_, b_] = xyz_to_rgb(self.xyz_to_rgb, x, y, z);

            **r = r_;
            **g = g_;
            **b = b_;
        }

        self.dst_transfer.linear_to_scaled_v(&mut i);

        block
    }
}
