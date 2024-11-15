use crate::{infer, AnySlice, ColorInfo, ConvertError, CropError, Cropped, PixelFormat, Window};
use std::error::Error;
use std::fmt;
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
    fn bounds_check(&self) -> bool {
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
impl Image<Vec<u8>> {
    pub fn blank(format: PixelFormat, width: usize, height: usize, color: ColorInfo) -> Self {
        Self::new(
            format,
            infer(
                format,
                vec![0u8; format.buffer_size(width, height)],
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

pub(crate) fn read_planes<'a, const N: usize>(
    mut iter: impl Iterator<Item = (&'a [u8], usize)>,
    format: PixelFormat,
) -> Result<[(&'a [u8], usize); N], ConvertError> {
    let mut out: [(&'a [u8], usize); N] = [(&[], 0); N];

    for out in &mut out {
        *out = iter
            .next()
            .ok_or(ConvertError::InvalidPlanesForPixelFormat(format))?;
    }

    Ok(out)
}

pub(crate) fn read_planes_mut<'a, const N: usize>(
    mut iter: impl Iterator<Item = (&'a mut [u8], usize)>,
    format: PixelFormat,
) -> Result<[(&'a mut [u8], usize); N], ConvertError> {
    let mut out: [MaybeUninit<(&'a mut [u8], usize)>; N] = [const { MaybeUninit::uninit() }; N];

    for out in &mut out {
        out.write(
            iter.next()
                .ok_or(ConvertError::InvalidPlanesForPixelFormat(format))?,
        );
    }

    Ok(out.map(|plane| unsafe { plane.assume_init() }))
}
