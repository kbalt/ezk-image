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
