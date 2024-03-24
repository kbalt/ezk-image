use crate::vector::Vector;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbaPixel<V> {
    pub(crate) r: V,
    pub(crate) g: V,
    pub(crate) b: V,
    pub(crate) a: V,
}

impl<V: Vector> RgbaPixel<V> {
    pub(crate) unsafe fn from_loaded8<const REVERSE: bool>(r: V, g: V, b: V, a: V) -> Self {
        let r = r.vdivf(255.0);
        let g = g.vdivf(255.0);
        let b = b.vdivf(255.0);
        let a = a.vdivf(255.0);

        if REVERSE {
            Self { r: b, g, b: r, a }
        } else {
            Self { r, g, b, a }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbaBlock<V> {
    pub(crate) rgba00: RgbaPixel<V>,
    pub(crate) rgba01: RgbaPixel<V>,
    pub(crate) rgba10: RgbaPixel<V>,
    pub(crate) rgba11: RgbaPixel<V>,
}

pub(crate) trait RgbaBlockVisitorImpl<V> {
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbaBlock<V>);
}

platform_trait!(RgbaBlockVisitor:RgbaBlockVisitorImpl);
