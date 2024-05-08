use crate::{planes::AnySlice, ColorInfo, PixelFormat, PixelFormatPlanes, Window};

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("width or height must not be zero")]
    InvalidDimensions,

    #[error("plane size does not match the provided dimensions and pixel format")]
    InvalidPlaneSize,
}

#[derive(Debug, thiserror::Error)]
#[error("Window position and/or size does not fit in the Image")]
pub struct ImageWindowError;

#[derive(Debug, Clone, Copy)]
pub struct Image<S: AnySlice> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: PixelFormatPlanes<S>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
    pub(crate) bits_per_component: usize,

    pub(crate) window: Option<Window>,
}

impl<S: AnySlice> Image<S> {
    pub fn new(
        format: PixelFormat,
        planes: PixelFormatPlanes<S>,
        width: usize,
        height: usize,
        color: ColorInfo,
        bits_per_component: usize,
    ) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions);
        }

        if !planes.bounds_check(width, height) {
            return Err(ImageError::InvalidPlaneSize);
        }

        Ok(Self {
            format,
            planes,
            width,
            height,
            color,
            bits_per_component,
            window: None,
        })
    }

    pub fn with_window(mut self, window: Window) -> Result<Self, ImageWindowError> {
        self.set_window(window)?;
        Ok(self)
    }

    pub fn set_window(&mut self, window: Window) -> Result<(), ImageWindowError> {
        if (window.x + window.width > self.width) || (window.y + window.height > self.height) {
            return Err(ImageWindowError);
        }

        self.window = Some(window);

        Ok(())
    }
}
