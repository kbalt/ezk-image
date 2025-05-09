#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]
#![cfg_attr(
    feature = "unstable",
    feature(stdarch_x86_avx512, avx512_target_feature)
)]

use formats::*;

mod color;
mod copy;
mod crop;
mod formats;
mod image;
mod image_traits;
#[cfg(feature = "multi-thread")]
mod multi_thread;
mod pixel_format;
mod plane_decs;
mod planes;
mod primitive;
#[cfg(feature = "resize")]
pub mod resize;
pub(crate) mod util;
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
pub use crop::{CropError, Cropped, Window};
pub use image::{Image, ImageError};
pub use image_traits::{ImageMut, ImageRef, ImageRefExt};
#[cfg(feature = "multi-thread")]
pub use multi_thread::convert_multi_thread;
pub use pixel_format::{BoundsCheckError, PixelFormat};
pub use planes::*;

/// Errors that may occur when trying to convert an image
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    #[error("image dimensions are not divisible by 2")]
    OddImageDimensions,

    #[error("source image has different size than destination image")]
    MismatchedImageSize,

    #[error("invalid color info for pixel format")]
    InvalidColorInfo,

    #[error(transparent)]
    BoundsCheck(#[from] BoundsCheckError),

    #[error(transparent)]
    InvalidNumberOfPlanes(#[from] InvalidNumberOfPlanesError),
}

/// Convert pixel-format and color from the src-image to the specified dst-image.
///
/// The given images (or at least their included window) must have dimensions (width, height) divisible by 2.
#[inline(never)]
pub fn convert(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
    verify_input_windows(src, dst)?;

    if src.format() == dst.format() && src.color() == dst.color() {
        // No color or pixel conversion needed just copy it over
        return copy(src, dst);
    }

    let src_color = src.color();
    let dst_color = dst.color();

    let reader = read_any_to_rgba(src)?;

    if need_transfer_and_primaries_convert(&src_color, &dst_color) {
        let reader = TransferAndPrimariesConvert::new(&src_color, &dst_color, reader);

        rgba_to_any(dst, reader)
    } else {
        rgba_to_any(dst, reader)
    }
}

#[inline(never)]
fn read_any_to_rgba<'a>(
    src: &'a dyn ImageRef,
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
fn rgba_to_any(dst: &mut dyn ImageMut, reader: impl RgbaSrc) -> Result<(), ConvertError> {
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
fn verify_input_windows(src: &dyn ImageRef, dst: &dyn ImageMut) -> Result<(), ConvertError> {
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
