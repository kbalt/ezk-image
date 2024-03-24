use super::rgb::{RgbBlock, RgbBlockVisitorImpl};
use super::rgba::{RgbaBlock, RgbaBlockVisitorImpl};
use crate::color::{ColorInfo, ColorOps};
use crate::vector::Vector;

pub(crate) struct RgbTransferAndPrimariesConvert<Vis> {
    src_color: ColorOps,
    dst_color: ColorOps,

    passthrough: bool,

    visitor: Vis,
}

impl<Vis> RgbTransferAndPrimariesConvert<Vis> {
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
            let x = r.vmulf(fw[0][0]).vadd(g.vmulf(fw[1][0])).vadd(b.vmulf(fw[2][0]));
            let y = r.vmulf(fw[0][1]).vadd(g.vmulf(fw[1][1])).vadd(b.vmulf(fw[2][1]));
            let z = r.vmulf(fw[0][2]).vadd(g.vmulf(fw[1][2])).vadd(b.vmulf(fw[2][2]));

            **r = x.vmulf(bw[0][0]).vadd(y.vmulf(bw[1][0])).vadd(z.vmulf(bw[2][0]));
            **g = x.vmulf(bw[0][1]).vadd(y.vmulf(bw[1][1])).vadd(z.vmulf(bw[2][1]));
            **b = x.vmulf(bw[0][2]).vadd(y.vmulf(bw[1][2])).vadd(z.vmulf(bw[2][2]));
        }
    }
}

impl<V, Vis> RgbBlockVisitorImpl<V> for RgbTransferAndPrimariesConvert<Vis>
where
    V: Vector,
    Vis: RgbBlockVisitorImpl<V>,
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, mut block: RgbBlock<V>) {
        if self.passthrough {
            self.visitor.visit(x, y, block);
            return;
        }

        let mut i = [
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

        V::color_ops(&self.src_color)
            .transfer
            .scaled_to_linear12(&mut i);

        self.convert_primaries(&mut i);

        V::color_ops(&self.dst_color)
            .transfer
            .linear_to_scaled12(&mut i);

        self.visitor.visit(x, y, block);
        self.visitor.visit(x, y, block);
    }
}

impl<V, Vis> RgbaBlockVisitorImpl<V> for RgbTransferAndPrimariesConvert<Vis>
where
    V: Vector,
    Vis: RgbaBlockVisitorImpl<V>,
{
    #[inline(always)]
    unsafe fn visit(&mut self, x: usize, y: usize, mut block: RgbaBlock<V>) {
        if self.passthrough {
            self.visitor.visit(x, y, block);
            return;
        }

        let mut i = [
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

        V::color_ops(&self.src_color)
            .transfer
            .scaled_to_linear12(&mut i);

        self.convert_primaries(&mut i);

        V::color_ops(&self.dst_color)
            .transfer
            .linear_to_scaled12(&mut i);

        self.visitor.visit(x, y, block);
    }
}
