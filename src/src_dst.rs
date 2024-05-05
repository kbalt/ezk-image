use crate::{primitive::Primitive, ColorInfo, PixelFormat, PixelFormatPlanes, Rect};

/// Describes an immutable image buffer used as source for conversions
#[derive(Debug, Clone, Copy)]
pub struct Source<'a, P: Primitive> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<&'a [P]>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a, P: Primitive> Source<'a, P> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<&'a [P]>,
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

/// Describes a mutable image buffer used as destination for conversions
pub struct Destination<'a, P: Primitive> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<&'a mut [P]>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a, P: Primitive> Destination<'a, P> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<&'a mut [P]>,
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
