use crate::{AnySlice, ColorInfo, ConvertError, PixelFormat};
use std::error::Error;
use std::fmt;
use std::mem::MaybeUninit;

pub(crate) fn read_planes<'a, const N: usize>(
    mut iter: impl Iterator<Item = &'a [u8]>,
    format: PixelFormat,
) -> Result<[&'a [u8]; N], ConvertError> {
    let mut out: [&'a [u8]; N] = [&[]; N];

    for out in &mut out {
        *out = iter
            .next()
            .ok_or(ConvertError::InvalidPlanesForPixelFormat(format))?;
    }

    Ok(out)
}

pub(crate) fn read_planes_mut<'a, const N: usize>(
    mut iter: impl Iterator<Item = &'a mut [u8]>,
    format: PixelFormat,
) -> Result<[&'a mut [u8]; N], ConvertError> {
    let mut out: [MaybeUninit<&'a mut [u8]>; N] = [const { MaybeUninit::uninit() }; N];

    for out in &mut out {
        out.write(
            iter.next()
                .ok_or(ConvertError::InvalidPlanesForPixelFormat(format))?,
        );
    }

    Ok(out.map(|plane| unsafe { plane.assume_init() }))
}

pub trait ImageRef<'a> {
    fn format(&self) -> PixelFormat;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn strides(&self) -> &[usize];
    fn planes(&self) -> impl Iterator<Item = &[u8]>;

    fn color(&self) -> ColorInfo;

    fn bounds_check(&self) -> bool {
        self.format()
            .bounds_check(self.planes(), self.strides(), self.width(), self.height())
    }
}

pub trait ImageMut<'a>: ImageRef<'a> {
    fn planes_mut(&mut self) -> impl Iterator<Item = &mut [u8]>;
}

/// Raw image data with information about dimensions, cropping, colorimetry, bit depth and pixel format
///
/// Type parameter `S` can be any of `&[u8]`, `&[u16]`, `&mut [u8]` or `&mut [u16]`, referencing the raw image data.
#[derive(Debug, Clone)]
pub struct Image<S: AnySlice> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: Vec<S>,
    pub(crate) strides: Vec<usize>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
}

impl<S: AnySlice> Image<S> {
    /// Create a new image from all non-optional fields
    #[deny(clippy::arithmetic_side_effects)]
    pub fn new(
        format: PixelFormat,
        planes: Vec<S>,
        strides: Vec<usize>,
        width: usize,
        height: usize,
        color: ColorInfo,
    ) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions);
        }

        let this = Self {
            format,
            planes,
            strides,
            width,
            height,
            color,
        };

        Ok(this)
    }

    // Set a cropping window but in a builder pattern
    // pub fn with_window(mut self, window: Window) -> Result<Self, ImageWindowError> {
    //     self.set_window(window)?;

    //     Ok(self)
    // }

    // /// Set a cropping window
    // #[deny(clippy::arithmetic_side_effects)]
    // pub fn set_window(&mut self, window: Window) -> Result<(), ImageWindowError> {
    //     type Error = ImageWindowError;

    //     if (window.x.checked_add(window.width).ok_or(Error {})? > self.width)
    //         || (window.y.checked_add(window.height).ok_or(Error {})? > self.height)
    //     {
    //         return Err(ImageWindowError);
    //     }

    //     self.window = Some(window);

    //     Ok(())
    // }
}

impl<'a> ImageRef<'a> for Image<&'a [u8]> {
    fn format(&self) -> PixelFormat {
        self.format
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn strides(&self) -> &[usize] {
        &self.strides
    }

    fn planes(&self) -> impl Iterator<Item = &[u8]> {
        self.planes.iter().copied()
    }

    fn color(&self) -> ColorInfo {
        self.color
    }
}

impl<'a> ImageRef<'a> for Image<&'a mut [u8]> {
    fn format(&self) -> PixelFormat {
        self.format
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn strides(&self) -> &[usize] {
        &self.strides
    }

    fn planes(&self) -> impl Iterator<Item = &[u8]> {
        self.planes.iter().map(|plane| &**plane)
    }

    fn color(&self) -> ColorInfo {
        self.color
    }
}

impl<'a> ImageMut<'a> for Image<&'a mut [u8]> {
    fn planes_mut(&mut self) -> impl Iterator<Item = &mut [u8]> {
        self.planes.iter_mut().map(|plane| &mut **plane)
    }
}

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

/// Cropping window of an [`Image`]
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
