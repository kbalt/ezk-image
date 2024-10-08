#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]
#![cfg_attr(
    feature = "unstable",
    feature(stdarch_x86_avx512, avx512_target_feature)
)]

use formats::*;
use primitive::PrimitiveInternal;
use std::{error::Error, fmt};

mod color;
mod copy;
mod formats;
mod image;
#[cfg(feature = "multi-thread")]
mod multi_thread;
mod planes;
mod primitive;
#[cfg(feature = "resize")]
pub mod resize;
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
#[doc(hidden)]
pub use copy::copy;
pub use image::{Image, ImageError, ImageWindowError, Window};
#[cfg(feature = "multi-thread")]
pub use multi_thread::convert_multi_thread;
pub use planes::PixelFormatPlanes;
pub use primitive::Primitive;

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// YUV with U and V sub-sampled in the vertical and horizontal dimension
    ///
    /// 3 Planes Y, U and V
    I420,

    /// YUV with U and V sub-sampled in the horizontal dimension
    ///
    /// 3 Planes Y, U and V
    I422,

    /// YUV
    ///
    /// 3 Planes Y, U and V
    I444,

    /// YUV with U and V sub-sampled in the vertical and horizontal dimension
    ///
    /// 2 Planes Y and UV interleaved
    NV12,

    /// YUV with U and V sub-sampled in the horizontal dimension
    ///
    /// 1 Plane, YUYV
    YUYV,

    /// RGBA
    ///
    /// 1 Plane RGBA interleaved
    RGBA,

    /// BGRA
    ///
    /// 1 Plane BGRA interleaved
    BGRA,

    /// RGB
    ///
    /// 1 Plane RGB interleaved
    RGB,

    /// BGR
    ///
    /// 1 Plane BGR interleaved
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
            RGBA | BGRA => width.strict_mul_(height).strict_mul_(4),
            RGB | BGR => width.strict_mul_(height).strict_mul_(3),
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
        }
    }
}

impl Error for ConvertError {}

/// Convert pixel-format and color from the src-image to the specified dst-image.
///
/// The given images (or at least their included window) must have dimensions (width, height) divisible by 2.
#[inline(never)]
pub fn convert<SP, DP>(src: Image<&[SP]>, dst: Image<&mut [DP]>) -> Result<(), ConvertError>
where
    SP: Primitive,
    DP: Primitive,
{
    get_and_verify_input_windows(&src, &dst)?;

    if src.format == dst.format && src.color == dst.color {
        // No color or pixel conversion needed just copy it over
        return convert_same_color_and_pixel_format(src, dst);
    }

    let reader: Box<dyn DynRgbaReader> = read_any_to_rgba(&src)?;

    if need_transfer_and_primaries_convert(&src.color, &dst.color) {
        let reader = TransferAndPrimariesConvert::new(&src.color, &dst.color, reader);

        rgba_to_any(dst, reader)
    } else {
        rgba_to_any(dst, reader)
    }
}

#[inline(never)]
fn convert_same_color_and_pixel_format<SP, DP>(
    src: Image<&[SP]>,
    dst: Image<&mut [DP]>,
) -> Result<(), ConvertError>
where
    SP: Primitive,
    DP: Primitive,
{
    assert_eq!(src.format, dst.format);

    match src.format {
        PixelFormat::I420 => I420Writer::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            I420Reader::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::I422 => I422Writer::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            I422Reader::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::I444 => I444Writer::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            I444Reader::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::NV12 => NV12Writer::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            NV12Reader::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::YUYV => YUYVWriter::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            YUYVReader::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::RGBA => RgbaWriter::<false, _, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbaReader::<false, _>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::BGRA => RgbaWriter::<true, _, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbaReader::<true, _>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::RGB => RgbWriter::<false, _, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbReader::<false, _>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
        PixelFormat::BGR => RgbWriter::<true, _, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbReader::<true, _>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        ),
    }
}

#[inline(never)]
fn read_any_to_rgba<'src, P>(
    src: &Image<&'src [P]>,
) -> Result<Box<dyn DynRgbaReader + 'src>, ConvertError>
where
    P: PrimitiveInternal,
{
    match src.format {
        PixelFormat::I420 => Ok(Box::new(I420ToRgb::new(
            &src.color,
            I420Reader::<P>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        )?)),
        PixelFormat::I422 => Ok(Box::new(I422ToRgb::new(
            &src.color,
            I422Reader::<P>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        )?)),
        PixelFormat::I444 => Ok(Box::new(I444ToRgb::new(
            &src.color,
            I444Reader::<P>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        )?)),
        PixelFormat::NV12 => Ok(Box::new(I420ToRgb::new(
            &src.color,
            NV12Reader::<P>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        )?)),
        PixelFormat::YUYV => Ok(Box::new(I422ToRgb::new(
            &src.color,
            YUYVReader::<P>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            )?,
        )?)),
        PixelFormat::RGBA => Ok(Box::new(RgbaReader::<false, P>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )?)),
        PixelFormat::BGRA => Ok(Box::new(RgbaReader::<true, P>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )?)),
        PixelFormat::RGB => Ok(Box::new(RgbReader::<false, P>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )?)),
        PixelFormat::BGR => Ok(Box::new(RgbReader::<true, P>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )?)),
    }
}

#[inline(never)]
fn rgba_to_any<'src, DP>(
    dst: Image<&mut [DP]>,
    reader: impl RgbaSrc + 'src,
) -> Result<(), ConvertError>
where
    DP: PrimitiveInternal,
{
    match dst.format {
        PixelFormat::I420 => I420Writer::<DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader)?,
        ),
        PixelFormat::I422 => I422Writer::<DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI422::new(&dst.color, reader)?,
        ),
        PixelFormat::I444 => I444Writer::<DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI444::new(&dst.color, reader)?,
        ),
        PixelFormat::NV12 => NV12Writer::<DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader)?,
        ),
        PixelFormat::YUYV => YUYVWriter::<DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI422::new(&dst.color, reader)?,
        ),
        PixelFormat::RGBA => RgbaWriter::<false, DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGRA => RgbaWriter::<true, DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::RGB => RgbWriter::<false, DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGR => RgbWriter::<true, DP, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
    }
}

/// Verify that the input values are all valid and safe to move on to
#[deny(clippy::arithmetic_side_effects)]
fn get_and_verify_input_windows<SP: Primitive, DP: Primitive>(
    src: &Image<&[SP]>,
    dst: &Image<&mut [DP]>,
) -> Result<(Window, Window), ConvertError> {
    let src_window = src.window.unwrap_or(Window {
        x: 0,
        y: 0,
        width: src.width,
        height: src.height,
    });

    let dst_window = dst.window.unwrap_or(Window {
        x: 0,
        y: 0,
        width: dst.width,
        height: dst.height,
    });

    // Src and Dst window must be the same size
    if src_window.width != dst_window.width || src_window.height != dst_window.height {
        return Err(ConvertError::MismatchedImageSize);
    }

    // Src and Dst window must have even dimensions
    if src_window.width % 2 == 1 || src_window.height % 2 == 1 {
        return Err(ConvertError::OddImageDimensions);
    }

    Ok((src_window, dst_window))
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
