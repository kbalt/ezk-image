use crate::{planes::AnySlice, ColorInfo, PixelFormat, PixelFormatPlanes};
use std::error::Error;
use std::fmt;

/// Everything that can go wrong when constructing an [`Image`]
#[derive(Debug, PartialEq)]
pub enum ImageError {
    InvalidDimensions,
    InvalidPlaneSize,
}

impl fmt::Display for ImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::InvalidDimensions => write!(f, "width or height must not be zero"),
            ImageError::InvalidPlaneSize => write!(
                f,
                "plane size does not match the provided dimensions and pixel format"
            ),
        }
    }
}

impl Error for ImageError {}

/// Error indicating an invalid [`Window`]
#[derive(Debug)]
pub struct ImageWindowError;

impl fmt::Display for ImageWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "window position and/or size does not fit in the image")
    }
}

impl Error for ImageWindowError {}

/// Raw image data with information about dimensions, cropping, colorimetry, bit depth and pixel format
///
/// Type parameter `S` can be any of `&[u8]`, `&[u16]`, `&mut [u8]` or `&mut [u16]`, referencing the raw image data.
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
    /// Create a new image from all non-optional fields
    #[deny(clippy::arithmetic_side_effects)]
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

    /// Set a cropping window but in a builder pattern
    pub fn with_window(mut self, window: Window) -> Result<Self, ImageWindowError> {
        self.set_window(window)?;

        Ok(self)
    }

    /// Set a cropping window
    #[deny(clippy::arithmetic_side_effects)]
    pub fn set_window(&mut self, window: Window) -> Result<(), ImageWindowError> {
        type Error = ImageWindowError;

        if (window.x.checked_add(window.width).ok_or(Error {})? > self.width)
            || (window.y.checked_add(window.height).ok_or(Error {})? > self.height)
        {
            return Err(ImageWindowError);
        }

        self.window = Some(window);

        Ok(())
    }
}

/// Cropping window of an [`Image`]
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
