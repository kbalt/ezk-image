use super::rgb::{RgbBlock, RgbBlockVisitor};
use super::rgba::{RgbaBlock, RgbaBlockVisitor};
use crate::color::primaries::{rgb_to_xyz, xyz_to_rgb};
use crate::color::{ColorInfo, ColorOps};
use crate::vector::Vector;

pub(crate) struct TransferAndPrimariesConvert<Vis> {
    src_color: ColorOps,
    dst_color: ColorOps,

    passthrough: bool,

    visitor: Vis,
}

impl<Vis> TransferAndPrimariesConvert<Vis> {
    pub(crate) fn new(src_color: &ColorInfo, dst_color: &ColorInfo, visitor: Vis) -> Self {
        Self {
            src_color: ColorOps::from_info(src_color),
            dst_color: ColorOps::from_info(dst_color),
            passthrough: src_color.transfer == dst_color.transfer
                && src_color.primaries == dst_color.primaries,
            visitor,
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

impl<Vis: RgbBlockVisitor> RgbBlockVisitor for TransferAndPrimariesConvert<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, mut block: RgbBlock<V>) {
        if self.passthrough {
            self.visitor.visit(x, y, block);
            return;
        }

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

        self.visitor.visit(x, y, block);
    }
}

impl<Vis: RgbaBlockVisitor> RgbaBlockVisitor for TransferAndPrimariesConvert<Vis> {
    #[inline(always)]
    unsafe fn visit<V: Vector>(&mut self, x: usize, y: usize, mut block: RgbaBlock<V>) {
        if self.passthrough {
            self.visitor.visit(x, y, block);
            return;
        }

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

        self.visitor.visit(x, y, block);
    }
}
