use crate::{infer, BoundsCheckError, ColorInfo, CropError, Cropped, PixelFormat, Window};
use std::mem::MaybeUninit;

pub trait ImageRef {
    fn format(&self) -> PixelFormat;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)>;
    fn color(&self) -> ColorInfo;
}

pub trait ImageMut: ImageRef {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)>;
}

pub trait ImageRefExt: ImageRef {
    fn bounds_check(&self) -> Result<(), BoundsCheckError> {
        self.format()
            .bounds_check(self.planes(), self.width(), self.height())
    }

    fn crop(self, window: Window) -> Result<Cropped<Self>, CropError>
    where
        Self: Sized,
    {
        Cropped::new(self, window)
    }
}

impl<T: ImageRef> ImageRefExt for T {}

impl<T: ImageRef> ImageRef for &T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

impl<T: ImageRef> ImageRef for &mut T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

impl<T: ImageMut> ImageMut for &mut T {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        <T as ImageMut>::planes_mut(self)
    }
}

/// Raw image data with information about dimensions, cropping, colorimetry, bit depth and pixel format
///
/// Type parameter `S` can be any of `&[u8]`, `&[u16]`, `&mut [u8]` or `&mut [u16]`, referencing the raw image data.
#[derive(Debug, Clone)]
pub struct Image<S> {
    pub(crate) format: PixelFormat,
    pub(crate) planes: Vec<S>,
    pub(crate) strides: Vec<usize>,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: ColorInfo,
}
impl Image<Vec<u8>> {
    pub fn blank(format: PixelFormat, width: usize, height: usize, color: ColorInfo) -> Self {
        Self::new(
            format,
            infer(
                format,
                vec![0u8; dbg!(format.buffer_size(width, height))],
                width,
                height,
            ),
            format.packed_strides(width),
            width,
            height,
            color,
        )
        .unwrap()
    }
}

impl<S> Image<S>
where
    Image<S>: ImageRef,
{
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

        this.bounds_check()?;

        Ok(this)
    }
}

impl<'a> ImageRef for Image<&'a [u8]> {
    fn format(&self) -> PixelFormat {
        self.format
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        self.planes
            .iter()
            .copied()
            .zip(self.strides.iter().copied())
    }
    fn color(&self) -> ColorInfo {
        self.color
    }
}

impl<'a> ImageRef for Image<&'a mut [u8]> {
    fn format(&self) -> PixelFormat {
        self.format
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        self.planes
            .iter()
            .map(|plane| &**plane)
            .zip(self.strides.iter().copied())
    }
    fn color(&self) -> ColorInfo {
        self.color
    }
}

impl<'a> ImageMut for Image<&'a mut [u8]> {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        self.planes
            .iter_mut()
            .map(|plane| &mut **plane)
            .zip(self.strides.iter().copied())
    }
}

impl ImageRef for Image<Vec<u8>> {
    fn format(&self) -> PixelFormat {
        self.format
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        self.planes
            .iter()
            .map(|plane| &**plane)
            .zip(self.strides.iter().copied())
    }
    fn color(&self) -> ColorInfo {
        self.color
    }
}

impl ImageMut for Image<Vec<u8>> {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        self.planes
            .iter_mut()
            .map(|plane| &mut **plane)
            .zip(self.strides.iter().copied())
    }
}

/// Everything that can go wrong when constructing an [`Image`]
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("width or height must not be zero")]
    InvalidDimensions,

    #[error(transparent)]
    BoundsCheck(#[from] BoundsCheckError),
}
