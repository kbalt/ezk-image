#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]
#![cfg_attr(
    feature = "unstable",
    feature(stdarch_x86_avx512, avx512_target_feature)
)]

use formats::*;
use image::read_planes;
use std::{error::Error, fmt};

mod color;
// mod copy;
mod formats;
mod image;
// #[cfg(feature = "multi-thread")]
// mod multi_thread;
mod planes;
mod primitive;
// #[cfg(feature = "resize")]
// pub mod resize;
mod vector;

mod arch {
    #[cfg(target_arch = "x86")]
    pub(crate) use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    pub(crate) use std::arch::x86_64::*;

    #[cfg(target_arch = "aarch64")]
    pub(crate) use std::arch::aarch64::*;
    #[cfg(target_arch = "aarch64")]
    pub(crate) use std::arch::is_aarch64_feature_detected;
}

pub use color::{ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, RgbColorInfo, YuvColorInfo};
pub use planes::*;
// #[doc(hidden)]
// pub use copy::copy;
pub use image::{Image, ImageError, ImageMut, ImageRef, ImageRefExt, ImageWindowError, Window};
// #[cfg(feature = "multi-thread")]
// pub use multi_thread::convert_multi_thread;

// compile_error!("pointer arithmetic for u16 !!!");

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// Y, U and V planes, 4:2:0 sub sampling, 8 bits per sample
    I420,

    /// Y, U and V planes, 4:2:2 sub sampling, 8 bits per sample
    I422,

    /// Y, U and V planes, 4:4:4 sub sampling, 8 bits per sample
    I444,

    /// Y, U, and V planes, 4:2:0 sub sampling, 10 bits per sample
    I010,

    /// Y, U, and V planes, 4:2:0 sub sampling, 12 bits per sample
    I012,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    I210,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    I212,

    /// Y, U, and V planes, 4:4:4 sub sampling, 10 bits per sample
    I410,

    /// Y, U, and V planes, 4:4:4 sub sampling, 12 bits per sample
    I412,

    /// Y and interleaved UV planes, 4:2:0 sub sampling
    NV12,

    /// Single YUYV, 4:2:2 sub sampling
    YUYV,

    /// Single RGBA interleaved plane
    RGBA,

    /// Single BGRA interleaved plane
    BGRA,

    /// Single RGB interleaved plane
    RGB,

    /// Single BGR interleaved plane
    BGR,
}

impl PixelFormat {
    /// Calculate the required buffer size given the [`PixelFormat`] self and image dimensions (in pixel width, height).
    ///
    /// The size is the amount of primitives (u8, u16) so when allocating size this must be accounted for.
    #[deny(clippy::arithmetic_side_effects)]
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        use PixelFormat::*;

        match self {
            I420 | NV12 => (width.strict_mul_(height).strict_mul_(12)).div_ceil(8),
            I422 | YUYV => width.strict_mul_(height).strict_mul_(2),
            I444 => width.strict_mul_(height).strict_mul_(3),
            I010 | I012 => I420.buffer_size(width, height).strict_mul_(2),
            I210 | I212 => I422.buffer_size(width, height).strict_mul_(2),
            I410 | I412 => I444.buffer_size(width, height).strict_mul_(2),
            RGBA | BGRA => width.strict_mul_(height).strict_mul_(4),
            RGB | BGR => width.strict_mul_(height).strict_mul_(3),
        }
    }

    pub fn bounds_check<'a>(
        self,
        planes: impl Iterator<Item = &'a [u8]>,
        strides: &[usize],
        width: usize,
        height: usize,
    ) -> bool {
        use PixelFormat::*;
        match self {
            I420 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                // Y WIDTH * HEIGHT
                //
                // U (WIDTH / 2) * (HEIGHT / 2)
                // V (WIDTH / 2) * (HEIGHT / 2)

                assert!(width <= strides[0]);
                assert!(width / 2 <= strides[1]);
                assert!(width / 2 <= strides[2]);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * (height / 2);
                let v = planes[2].len() >= strides[2] * (height / 2);

                y && u && v
            }
            I422 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                assert!(width <= strides[0]);
                assert!(width <= strides[1] / 2);
                assert!(width <= strides[2] / 2);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * height;
                let v = planes[2].len() >= strides[2] * height;

                y && u && v
            }
            I444 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                assert!(width <= strides[0]);
                assert!(width <= strides[1]);
                assert!(width <= strides[2]);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * height;
                let v = planes[2].len() >= strides[2] * height;

                y && u && v
            }
            I010 | I012 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                assert!(width * 2 <= strides[0]);
                assert!(width * 2 <= strides[1] / 2);
                assert!(width * 2 <= strides[2] / 2);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * (height / 2);
                let v = planes[2].len() >= strides[2] * (height / 2);

                y && u && v
            }
            I210 | I212 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                assert!(width * 2 <= strides[0]);
                assert!(width * 2 <= strides[1] / 2);
                assert!(width * 2 <= strides[2] / 2);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * height;
                let v = planes[2].len() >= strides[2] * height;

                y && u && v
            }
            I410 | I412 => {
                let planes: [_; 3] = read_planes(planes, self).unwrap();

                assert!(width * 2 <= strides[0]);
                assert!(width * 2 <= strides[1]);
                assert!(width * 2 <= strides[2]);

                let y = planes[0].len() >= strides[0] * height;
                let u = planes[1].len() >= strides[1] * height;
                let v = planes[2].len() >= strides[2] * height;

                y && u && v
            }
            NV12 => {
                let planes: [_; 2] = read_planes(planes, self).unwrap();

                assert!(width <= strides[0]);
                assert!(width <= strides[1]);

                let y = planes[0].len() >= strides[0] * height;
                let uv = planes[1].len() >= strides[1] * (height / 2);

                y && uv
            }
            YUYV | RGBA | BGRA | RGB | BGR => {
                let planes: [_; 1] = read_planes(planes, self).unwrap();

                assert!(width <= strides[0]);

                planes[0].len() >= strides[0] * height
            }
        }
    }

    pub fn bits_per_component(&self) -> usize {
        match self {
            PixelFormat::I420 => 8,
            PixelFormat::I422 => 8,
            PixelFormat::I444 => 8,
            PixelFormat::I010 => 10,
            PixelFormat::I012 => 12,
            PixelFormat::I210 => 10,
            PixelFormat::I212 => 12,
            PixelFormat::I410 => 10,
            PixelFormat::I412 => 12,
            PixelFormat::NV12 => 8,
            PixelFormat::YUYV => 8,
            PixelFormat::RGBA => 8,
            PixelFormat::BGRA => 8,
            PixelFormat::RGB => 8,
            PixelFormat::BGR => 8,
        }
    }
}

/// Errors that may occur when trying to convert an image
#[derive(Debug, PartialEq)]
pub enum ConvertError {
    OddImageDimensions,
    MismatchedImageSize,
    InvalidColorInfo,
    InvalidPlanesForPixelFormat(PixelFormat),
    InvalidPlaneSizeForDimensions,
    InvalidStridesForPixelFormat(PixelFormat),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::OddImageDimensions => {
                write!(f, "image dimensions are not divisible by 2")
            }
            ConvertError::MismatchedImageSize => {
                write!(f, "source image has different size than destination image")
            }
            ConvertError::InvalidColorInfo => {
                write!(f, "invalid color info for pixel format")
            }
            ConvertError::InvalidPlanesForPixelFormat(format) => {
                write!(f, "provided planes mismatch with {format:?}")
            }
            ConvertError::InvalidPlaneSizeForDimensions => write!(
                f,
                "provided planes are too small for the given image dimensions"
            ),
            ConvertError::InvalidStridesForPixelFormat(pixel_format) => write!(
                f,
                "provided to few, or too many planes for the given pixel format {pixel_format:?}"
            ),
        }
    }
}

impl Error for ConvertError {}

/// Convert pixel-format and color from the src-image to the specified dst-image.
///
/// The given images (or at least their included window) must have dimensions (width, height) divisible by 2.
#[inline(never)]
pub fn convert(src: &impl ImageRef, dst: &mut impl ImageMut) -> Result<(), ConvertError> {
    verify_input_windows(src, dst)?;

    if src.format() == dst.format() && src.color() == dst.color() {
        // No color or pixel conversion needed just copy it over
        return convert_same_color_and_pixel_format(src, dst);
    }

    let src_color = src.color();
    let dst_color = dst.color();

    let reader: Box<dyn DynRgbaReader> = read_any_to_rgba(src)?;

    if need_transfer_and_primaries_convert(&src_color, &dst_color) {
        let reader = TransferAndPrimariesConvert::new(&src_color, &dst_color, reader);

        rgba_to_any(dst, reader)
    } else {
        rgba_to_any(dst, reader)
    }
}

#[inline(never)]
fn convert_same_color_and_pixel_format<'a, 'b>(
    src: &'a impl ImageRef,
    dst: &'b mut impl ImageMut,
) -> Result<(), ConvertError> {
    use PixelFormat::*;
    assert_eq!(src.format(), dst.format());

    match src.format() {
        I420 => I420Writer::<u8, _>::write(dst, I420Reader::<u8>::new(src)?),
        I422 => I422Writer::<u8, _>::write(dst, I422Reader::<u8>::new(src)?),
        I444 => I444Writer::<u8, _>::write(dst, I444Reader::<u8>::new(src)?),

        I010 | I012 => I420Writer::<u16, _>::write(dst, I420Reader::<u16>::new(src)?),
        I210 | I212 => I422Writer::<u16, _>::write(dst, I422Reader::<u16>::new(src)?),
        I410 | I412 => I444Writer::<u16, _>::write(dst, I444Reader::<u16>::new(src)?),

        NV12 => NV12Writer::<u8, _>::write(dst, NV12Reader::<u8>::new(src)?),
        YUYV => YUYVWriter::<u8, _>::write(dst, YUYVReader::<u8>::new(src)?),

        RGBA => RgbaWriter::<false, u8, _>::write(dst, RgbaReader::<false, u8>::new(src)?),
        BGRA => RgbaWriter::<true, u8, _>::write(dst, RgbaReader::<true, u8>::new(src)?),
        RGB => RgbWriter::<false, u8, _>::write(dst, RgbReader::<false, u8>::new(src)?),
        BGR => RgbWriter::<true, u8, _>::write(dst, RgbReader::<true, u8>::new(src)?),
    }
}

#[inline(never)]
fn read_any_to_rgba<'a>(
    src: &'a impl ImageRef,
) -> Result<Box<dyn DynRgbaReader + 'a>, ConvertError> {
    use PixelFormat::*;

    match src.format() {
        I420 => Ok(Box::new(I420ToRgb::new(
            &src.color(),
            I420Reader::<u8>::new(src)?,
        )?)),
        I422 => Ok(Box::new(I422ToRgb::new(
            &src.color(),
            I422Reader::<u8>::new(src)?,
        )?)),
        I444 => Ok(Box::new(I444ToRgb::new(
            &src.color(),
            I444Reader::<u8>::new(src)?,
        )?)),

        I010 | I012 => Ok(Box::new(I420ToRgb::new(
            &src.color(),
            I420Reader::<u16>::new(src)?,
        )?)),
        I210 | I212 => Ok(Box::new(I422ToRgb::new(
            &src.color(),
            I422Reader::<u16>::new(src)?,
        )?)),
        I410 | I412 => Ok(Box::new(I444ToRgb::new(
            &src.color(),
            I444Reader::<u16>::new(src)?,
        )?)),

        NV12 => Ok(Box::new(I420ToRgb::new(
            &src.color(),
            NV12Reader::<u8>::new(src)?,
        )?)),
        YUYV => Ok(Box::new(I422ToRgb::new(
            &src.color(),
            YUYVReader::<u8>::new(src)?,
        )?)),

        RGBA => Ok(Box::new(RgbaReader::<false, u8>::new(src)?)),
        BGRA => Ok(Box::new(RgbaReader::<true, u8>::new(src)?)),
        RGB => Ok(Box::new(RgbReader::<false, u8>::new(src)?)),
        BGR => Ok(Box::new(RgbReader::<true, u8>::new(src)?)),
    }
}

#[inline(never)]
fn rgba_to_any(dst: &mut impl ImageMut, reader: impl RgbaSrc) -> Result<(), ConvertError> {
    use PixelFormat::*;

    let dst_color = dst.color();

    match dst.format() {
        I420 => I420Writer::<u8, _>::write(dst, RgbToI420::new(&dst_color, reader)?),
        I422 => I422Writer::<u8, _>::write(dst, RgbToI422::new(&dst_color, reader)?),
        I444 => I444Writer::<u8, _>::write(dst, RgbToI444::new(&dst_color, reader)?),

        I010 | I012 => I420Writer::<u16, _>::write(dst, RgbToI420::new(&dst_color, reader)?),
        I210 | I212 => I422Writer::<u16, _>::write(dst, RgbToI422::new(&dst_color, reader)?),
        I410 | I412 => I444Writer::<u16, _>::write(dst, RgbToI444::new(&dst_color, reader)?),

        NV12 => NV12Writer::<u8, _>::write(dst, RgbToI420::new(&dst_color, reader)?),
        YUYV => YUYVWriter::<u8, _>::write(dst, RgbToI422::new(&dst_color, reader)?),

        RGBA => RgbaWriter::<false, u8, _>::write(dst, reader),
        BGRA => RgbaWriter::<true, u8, _>::write(dst, reader),
        RGB => RgbWriter::<false, u8, _>::write(dst, reader),
        BGR => RgbWriter::<true, u8, _>::write(dst, reader),
    }
}

/// Verify that the input values are all valid and safe to move on to
#[deny(clippy::arithmetic_side_effects)]
fn verify_input_windows(src: &impl ImageRef, dst: &impl ImageMut) -> Result<(), ConvertError> {
    // Src and Dst window must be the same size
    if src.width() != dst.width() || src.height() != dst.height() {
        return Err(ConvertError::MismatchedImageSize);
    }

    // Src and Dst window must have even dimensions
    if src.width() % 2 == 1 || src.height() % 2 == 1 {
        return Err(ConvertError::OddImageDimensions);
    }

    Ok(())
}

trait StrictApi {
    fn strict_add_(self, rhs: Self) -> Self;
    fn strict_mul_(self, rhs: Self) -> Self;
}

impl StrictApi for usize {
    #[track_caller]
    fn strict_add_(self, rhs: Self) -> Self {
        self.checked_add(rhs).expect("attempt to add with overflow")
    }

    #[track_caller]
    fn strict_mul_(self, rhs: Self) -> Self {
        self.checked_mul(rhs)
            .expect("attempt to multiply with overflow")
    }
}
