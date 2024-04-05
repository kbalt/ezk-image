use crate::{bits::Bits, ColorInfo, PixelFormat, PixelFormatPlanes, Rect};

/// Describes an immutable image buffer used as source for conversions
#[derive(Debug, Clone, Copy)]
pub struct Source<'a, B: Bits> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<&'a [B::Primitive]>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a, B: Bits> Source<'a, B> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<&'a [B::Primitive]>,
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
pub struct Destination<'a, B: Bits> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Rect>,
}

impl<'a, B: Bits> Destination<'a, B> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<&'a mut [B::Primitive]>,
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
