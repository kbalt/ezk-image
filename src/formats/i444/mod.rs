mod from_rgb;
mod read;
mod to_rgb;
mod write;

pub(crate) use from_rgb::RgbToI444Visitor;
pub(crate) use read::read_i444;
pub(crate) use to_rgb::I444ToRgbVisitor;
pub(crate) use write::I444Writer;

pub(crate) struct I444Block<V> {
    pub(crate) px00: I444Pixel<V>,
    pub(crate) px01: I444Pixel<V>,
    pub(crate) px10: I444Pixel<V>,
    pub(crate) px11: I444Pixel<V>,
}

pub(crate) struct I444Pixel<V> {
    pub(crate) y: V,
    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait I444VisitorImpl<V> {
    unsafe fn visit(&mut self, x: usize, y: usize, block: I444Block<V>);
}

platform_trait!(I444Visitor:I444VisitorImpl);
