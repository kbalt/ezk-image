mod from_rgb;
mod read;
mod to_rgb;
mod write;

pub(crate) use from_rgb::RgbToI420Visitor;
pub(crate) use read::read_i420;
pub(crate) use to_rgb::I420ToRgbVisitor;
pub(crate) use write::I420Writer;

pub(crate) struct I420Block<V> {
    pub(crate) y00: V,
    pub(crate) y01: V,
    pub(crate) y10: V,
    pub(crate) y11: V,

    pub(crate) u: V,
    pub(crate) v: V,
}

pub(crate) trait I420VisitorImpl<V> {
    unsafe fn visit(&mut self, x: usize, y: usize, block: I420Block<V>);
}

platform_trait!(I420Visitor:I420VisitorImpl);