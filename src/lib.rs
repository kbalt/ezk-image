#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]
#![allow(unsafe_op_in_unsafe_fn)]
// TODO: actually feature gating every single function and use is impossible right now, so until then warnings
// for unused items are only enabled when all-formats is activated
#![cfg_attr(not(feature = "all-formats"), allow(unused))]

#[cfg(not(any(
    feature = "I420",
    feature = "I422",
    feature = "I444",
    feature = "I010",
    feature = "I012",
    feature = "I210",
    feature = "I212",
    feature = "I410",
    feature = "I412",
    feature = "NV12",
    feature = "P010",
    feature = "P012",
    feature = "YUYV",
    feature = "RGBA",
    feature = "BGRA",
    feature = "ARGB",
    feature = "ABGR",
    feature = "RGB",
    feature = "BGR",
)))]
compile_error!("At least one image format feature must be enabled");

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
pub use image::{BufferKind, Image, ImageError};
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
        #[cfg(feature = "I420")]
        I420 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read3Plane::<u8>::new(src)?,
        )?)),
        #[cfg(feature = "I422")]
        I422 => Ok(Box::new(yuv422::ToRgb::new(
            &src.color(),
            yuv422::Read3Plane::<u8>::new(src)?,
        )?)),
        #[cfg(feature = "I444")]
        I444 => Ok(Box::new(yuv444::ToRgb::new(
            &src.color(),
            yuv444::Read3Plane::<u8>::new(src)?,
        )?)),
        #[cfg(feature = "I010")]
        I010 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "I012")]
        I012 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "I210")]
        I210 => Ok(Box::new(yuv422::ToRgb::new(
            &src.color(),
            yuv422::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "I212")]
        I212 => Ok(Box::new(yuv422::ToRgb::new(
            &src.color(),
            yuv422::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "I410")]
        I410 => Ok(Box::new(yuv444::ToRgb::new(
            &src.color(),
            yuv444::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "I412")]
        I412 => Ok(Box::new(yuv444::ToRgb::new(
            &src.color(),
            yuv444::Read3Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "NV12")]
        NV12 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read2Plane::<u8>::new(src)?,
        )?)),
        #[cfg(feature = "P010")]
        P010 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read2Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "P012")]
        P012 => Ok(Box::new(yuv420::ToRgb::new(
            &src.color(),
            yuv420::Read2Plane::<u16>::new(src)?,
        )?)),
        #[cfg(feature = "YUYV")]
        YUYV => Ok(Box::new(yuv422::ToRgb::new(
            &src.color(),
            yuv422::Read1Plane::<u8>::new(src)?,
        )?)),
        #[cfg(feature = "RGBA")]
        RGBA => Ok(Box::new(rgb::ReadRgba::<u8>::new(src)?)),
        #[cfg(feature = "BGRA")]
        BGRA => Ok(Box::new(rgb::ReadBgra::<u8>::new(src)?)),
        #[cfg(feature = "ARGB")]
        ARGB => Ok(Box::new(rgb::ReadArgb::<u8>::new(src)?)),
        #[cfg(feature = "ABGR")]
        ABGR => Ok(Box::new(rgb::ReadAbgr::<u8>::new(src)?)),
        #[cfg(feature = "RGB")]
        RGB => Ok(Box::new(rgb::ReadRgb::<u8>::new(src)?)),
        #[cfg(feature = "BGR")]
        BGR => Ok(Box::new(rgb::ReadBgr::<u8>::new(src)?)),
    }
}

#[inline(never)]
fn rgba_to_any(dst: &mut dyn ImageMut, reader: impl rgb::RgbaSrc) -> Result<(), ConvertError> {
    use PixelFormat::*;

    let color = dst.color();

    match dst.format() {
        #[cfg(feature = "I420")]
        I420 => yuv420::Write3Plane::<u8, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I422")]
        I422 => yuv422::Write3Plane::<u8, _>::write(dst, yuv422::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I444")]
        I444 => yuv444::Write3Plane::<u8, _>::write(dst, yuv444::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I010")]
        I010 => yuv420::Write3Plane::<u16, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I012")]
        I012 => yuv420::Write3Plane::<u16, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I210")]
        I210 => yuv422::Write3Plane::<u16, _>::write(dst, yuv422::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I212")]
        I212 => yuv422::Write3Plane::<u16, _>::write(dst, yuv422::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I410")]
        I410 => yuv444::Write3Plane::<u16, _>::write(dst, yuv444::FromRgb::new(&color, reader)?),
        #[cfg(feature = "I412")]
        I412 => yuv444::Write3Plane::<u16, _>::write(dst, yuv444::FromRgb::new(&color, reader)?),
        #[cfg(feature = "NV12")]
        NV12 => yuv420::Write2Plane::<u8, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "P010")]
        P010 => yuv420::Write2Plane::<u16, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "P012")]
        P012 => yuv420::Write2Plane::<u16, _>::write(dst, yuv420::FromRgb::new(&color, reader)?),
        #[cfg(feature = "YUYV")]
        YUYV => yuv422::Write1Plane::<u8, _>::write(dst, yuv422::FromRgb::new(&color, reader)?),
        #[cfg(feature = "RGBA")]
        RGBA => rgb::WriteRgba::<u8, _>::write(dst, reader),
        #[cfg(feature = "BGRA")]
        BGRA => rgb::WriteBgra::<u8, _>::write(dst, reader),
        #[cfg(feature = "ARGB")]
        ARGB => rgb::WriteArgb::<u8, _>::write(dst, reader),
        #[cfg(feature = "ABGR")]
        ABGR => rgb::WriteAbgr::<u8, _>::write(dst, reader),
        #[cfg(feature = "RGB")]
        RGB => rgb::WriteRgb::<u8, _>::write(dst, reader),
        #[cfg(feature = "BGR")]
        BGR => rgb::WriteBgr::<u8, _>::write(dst, reader),
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
