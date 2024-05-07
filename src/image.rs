use crate::{planes::AnySlice, ColorInfo, PixelFormat, PixelFormatPlanes, Rect};

#[derive(Debug, Clone, Copy)]
pub struct Image<S: AnySlice> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<S>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Rect>,
}

impl<S: AnySlice> Image<S> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<S>,
        width: usize,
        height: usize,
        color: ColorInfo,
        bits_per_component: usize,
    ) -> Self {
        assert!(planes.bounds_check(width, height));

        Self {
            format,
            planes,
            width,
            height,
            color,
            bits_per_component,
            window: None,
        }
    }

    pub fn with_window(mut self, window: Rect) -> Self {
        self.set_window(window);
        self
    }

    pub fn set_window(&mut self, window: Rect) {
        assert!(window.x + window.width <= self.width);
        assert!(window.y + window.height <= self.height);

        self.window = Some(window)
    }
}
