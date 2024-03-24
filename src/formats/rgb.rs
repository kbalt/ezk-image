use crate::vector::Vector;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbPixel<V> {
    pub(crate) r: V,
    pub(crate) g: V,
    pub(crate) b: V,
}

impl<V: Vector> RgbPixel<V> {
    pub(crate) unsafe fn from_loaded8<const REVERSE: bool>(r: V, g: V, b: V) -> Self {
        let r = r.vdivf(255.0);
        let g = g.vdivf(255.0);
        let b = b.vdivf(255.0);

        if REVERSE {
            Self { r: b, g, b: r }
        } else {
            Self { r, g, b }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RgbBlock<V> {
    pub(crate) rgb00: RgbPixel<V>,
    pub(crate) rgb01: RgbPixel<V>,
    pub(crate) rgb10: RgbPixel<V>,
    pub(crate) rgb11: RgbPixel<V>,
}

pub(crate) trait RgbBlockVisitorImpl<V> {
    unsafe fn visit(&mut self, x: usize, y: usize, block: RgbBlock<V>);
}

platform_trait!(RgbBlockVisitor:RgbBlockVisitorImpl);
