#![doc = include_str!("../README.md")]

use bits::BitsInternal;
use formats::*;

pub use bits::{Bits, U16BE, U16LE, U8};
pub use color::{ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer};
#[cfg(feature = "multi-thread")]
pub use multi_thread::convert_multi_thread;
pub use planes::PixelFormatPlanes;
pub use src_dst::{Destination, Source};

mod bits;
mod color;
mod endian;
mod formats;
#[cfg(feature = "multi-thread")]
mod multi_thread;
mod planes;
mod src_dst;
mod vector;

mod arch {
    #[cfg(target_arch = "x86")]
    pub use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    pub use std::arch::x86_64::*;

    #[cfg(target_arch = "aarch64")]
    pub use std::arch::aarch64::*;
    #[cfg(target_arch = "aarch64")]
    pub use std::arch::is_aarch64_feature_detected;
}

fn max_value_for_bits(bits: usize) -> f32 {
    ((1 << bits) - 1) as f32
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Verify that the input values are all valid and safe to move on to
fn verify_input_windows_same_size<SB: Bits, DB: Bits>(
    src: &Source<'_, SB>,
    dst: &Destination<'_, DB>,
) -> (Rect, Rect) {
    let src_window = src.window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src.width,
        height: src.height,
    });

    let dst_window = dst.window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: dst.width,
        height: dst.height,
    });

    // Src and Dst window must be the same size
    assert_eq!(src_window.width, dst_window.width);
    assert_eq!(src_window.height, dst_window.height);

    (src_window, dst_window)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// 3 Planes, Y, U, V
    ///
    /// 4x Y, 1x U, 1x V
    I420,

    /// 3 Planes, Y, U, V
    ///
    /// 4x Y, 2x U, 2x V
    I422,

    /// 3 Planes, Y, U, V
    ///
    /// 1x Y, 1x U, 1x V
    I444,

    /// 2 Planes, Y, U & V interleaved
    ///
    /// 4x Y, 1x U & V
    NV12,

    /// 1 Plane 4 primitives R, G, B, A
    RGBA,

    /// 1 Plane 4 primitives B, G, R, A
    BGRA,

    /// 1 Plane 3 primitives R, G, B
    RGB,

    /// 1 Plane 3 primitives B, G, R
    BGR,
}

impl PixelFormat {
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        use PixelFormat::*;

        match self {
            I420 | NV12 => (width * height * 12).div_ceil(8),
            I422 => (width * height * 16).div_ceil(8),
            I444 => width * height * 3,
            RGBA | BGRA => width * height * 4,
            RGB | BGR => width * height * 3,
        }
    }
}

#[allow(private_bounds)]
#[inline(never)]
pub fn convert<SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    verify_input_windows_same_size(&src, &dst);

    let reader: Box<dyn DynRgbaReader> = read_any_to_rgba(&src);

    if need_transfer_and_primaries_convert(&src.color, &dst.color) {
        let reader = TransferAndPrimariesConvert::new(&src.color, &dst.color, reader);

        rgba_to_any(dst, reader);
    } else {
        rgba_to_any(dst, reader);
    }
}

#[inline(never)]
fn read_any_to_rgba<'src, SB>(src: &Source<'src, SB>) -> Box<dyn DynRgbaReader + 'src>
where
    SB: BitsInternal,
{
    match src.format {
        PixelFormat::I420 => Box::new(I420ToRgb::new(
            &src.color,
            I420Reader::<SB>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            ),
        )),
        PixelFormat::I422 => Box::new(I422ToRgb::new(
            &src.color,
            I422Reader::<SB>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            ),
        )),
        PixelFormat::I444 => Box::new(I444ToRgb::new(
            &src.color,
            I444Reader::<SB>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            ),
        )),
        PixelFormat::NV12 => Box::new(I420ToRgb::new(
            &src.color,
            NV12Reader::<SB>::new(
                src.width,
                src.height,
                src.planes,
                src.bits_per_component,
                src.window,
            ),
        )),
        PixelFormat::RGBA => Box::new(RgbaReader::<false, SB>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )),
        PixelFormat::BGRA => Box::new(RgbaReader::<true, SB>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )),
        PixelFormat::RGB => Box::new(RgbReader::<false, SB>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )),
        PixelFormat::BGR => Box::new(RgbReader::<true, SB>::new(
            src.width,
            src.height,
            src.planes,
            src.bits_per_component,
            src.window,
        )),
    }
}

#[inline(never)]
fn rgba_to_any<'src, DB>(dst: Destination<'_, DB>, reader: impl RgbaSrc + 'src)
where
    DB: BitsInternal,
{
    match dst.format {
        PixelFormat::I420 => I420Writer::<DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader),
        ),
        PixelFormat::I422 => I422Writer::<DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI422::new(&dst.color, reader),
        ),
        PixelFormat::I444 => I444Writer::<DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI444::new(&dst.color, reader),
        ),
        PixelFormat::NV12 => NV12Writer::<DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader),
        ),
        PixelFormat::RGBA => RgbaWriter::<false, DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGRA => RgbaWriter::<true, DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::RGB => RgbWriter::<false, DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGR => RgbWriter::<true, DB, _>::write(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
    }
}

#[allow(private_bounds)]
pub fn scale<B>(src: Source<'_, B>, dst: Destination<'_, B>)
where
    B: BitsInternal,

    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, B::FirPixel1>>
        + From<fir::ImageView<'a, B::FirPixel2>>
        + From<fir::ImageView<'a, B::FirPixel3>>
        + From<fir::ImageView<'a, B::FirPixel4>>,

    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, B::FirPixel1>>
        + From<fir::ImageViewMut<'a, B::FirPixel2>>
        + From<fir::ImageViewMut<'a, B::FirPixel3>>
        + From<fir::ImageViewMut<'a, B::FirPixel4>>,
{
    assert_eq!(src.format, dst.format);

    let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Bilinear));

    match (src.planes, dst.planes) {
        (
            PixelFormatPlanes::I420 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I420 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_v,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );
        }
        (
            PixelFormatPlanes::I422 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I422 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32 / 2,
                dst_v,
                dst.width as u32,
                dst.height as u32 / 2,
            );
        }
        (
            PixelFormatPlanes::I444 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I444 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32,
                dst_u,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32,
                dst_v,
                dst.width as u32,
                dst.height as u32,
            );
        }
        (
            PixelFormatPlanes::NV12 {
                y: src_y,
                uv: src_uv,
            },
            PixelFormatPlanes::NV12 {
                y: dst_y,
                uv: dst_uv,
            },
        ) => {
            resize_plane::<_, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<_, B::FirPixel2>(
                &mut resizer,
                src_uv,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_uv,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );
        }
        (PixelFormatPlanes::RGB(src_rgb), PixelFormatPlanes::RGB(dst_rgb)) => {
            resize_plane::<_, B::FirPixel3>(
                &mut resizer,
                src_rgb,
                src.width as u32,
                src.height as u32,
                dst_rgb,
                dst.width as u32,
                dst.height as u32,
            );
        }
        (PixelFormatPlanes::RGBA(src_rgba), PixelFormatPlanes::RGBA(dst_rgba)) => {
            resize_plane::<_, B::FirPixel4>(
                &mut resizer,
                src_rgba,
                src.width as u32,
                src.height as u32,
                dst_rgba,
                dst.width as u32,
                dst.height as u32,
            );
        }
        _ => unreachable!(),
    }
}

fn resize_plane<P, Px>(
    resizer: &mut fir::Resizer,

    src: &[P],
    src_width: u32,
    src_height: u32,

    dst: &mut [P],
    dst_width: u32,
    dst_height: u32,
) where
    Px: fir::pixels::PixelExt,
    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, Px>>,
    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, Px>>,
{
    let src_slice = unsafe {
        std::slice::from_raw_parts(src.as_ptr() as *const u8, std::mem::size_of_val(src))
    };
    let dst_slice = unsafe {
        std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut u8, std::mem::size_of_val(dst))
    };

    let src = fir::ImageView::<Px>::from_buffer(
        src_width.try_into().unwrap(),
        src_height.try_into().unwrap(),
        src_slice,
    )
    .unwrap();

    let dst = fir::ImageViewMut::<Px>::from_buffer(
        dst_width.try_into().unwrap(),
        dst_height.try_into().unwrap(),
        dst_slice,
    )
    .unwrap();

    resizer
        .resize(
            &fir::DynamicImageView::from(src),
            &mut fir::DynamicImageViewMut::from(dst),
        )
        .unwrap();
}
