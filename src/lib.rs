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

macro_rules! read_i420 {
    ($src:ident) => {
        I420ToRgb::new(
            &$src.color,
            I420Reader::<SB>::new(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_component,
                $src.window,
            ),
        )
    };
}

macro_rules! read_i422 {
    ($src:ident) => {
        I422ToRgb::new(
            &$src.color,
            I422Reader::<SB>::new(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_component,
                $src.window,
            ),
        )
    };
}

macro_rules! read_i444 {
    ($src:ident) => {
        I444ToRgb::new(
            &$src.color,
            I444Reader::<SB>::new(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_component,
                $src.window,
            ),
        )
    };
}

macro_rules! read_nv12 {
    ($src:ident) => {
        I420ToRgb::new(
            &$src.color,
            NV12Reader::<SB>::new(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_component,
                $src.window,
            ),
        )
    };
}

macro_rules! read_rgb {
    ($src:ident) => {
        RgbReader::<false, SB>::new(
            $src.width,
            $src.height,
            $src.planes,
            $src.bits_per_component,
            $src.window,
        )
    };
}

macro_rules! read_bgr {
    ($src:ident) => {
        RgbReader::<true, SB>::new(
            $src.width,
            $src.height,
            $src.planes,
            $src.bits_per_component,
            $src.window,
        )
    };
}

macro_rules! read_rgba {
    ($src:ident) => {
        RgbaReader::<false, SB>::new(
            $src.width,
            $src.height,
            $src.planes,
            $src.bits_per_component,
            $src.window,
        )
    };
}

macro_rules! read_bgra {
    ($src:ident) => {
        RgbaReader::<true, SB>::new(
            $src.width,
            $src.height,
            $src.planes,
            $src.bits_per_component,
            $src.window,
        )
    };
}

#[allow(private_bounds)]
#[inline(never)]
pub fn convert<SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    verify_input_windows_same_size(&src, &dst);

    let reader: Box<dyn DynRgbaReader> = match src.format {
        PixelFormat::I420 => Box::new(read_i420!(src)),
        PixelFormat::I422 => Box::new(read_i422!(src)),
        PixelFormat::I444 => Box::new(read_i444!(src)),
        PixelFormat::NV12 => Box::new(read_nv12!(src)),
        PixelFormat::RGBA => Box::new(read_rgba!(src)),
        PixelFormat::BGRA => Box::new(read_bgra!(src)),
        PixelFormat::RGB => Box::new(read_rgb!(src)),
        PixelFormat::BGR => Box::new(read_bgr!(src)),
    };

    if need_transfer_and_primaries_convert(&src.color, &dst.color) {
        let reader = TransferAndPrimariesConvert::new(&src.color, &dst.color, reader);

        rgba_to_any(src, dst, reader);
    } else {
        rgba_to_any(src, dst, reader);
    }
}

fn rgba_to_any<'src, SB, DB>(
    src: Source<'src, SB>,
    dst: Destination<'_, DB>,
    reader: impl RgbaSrc + 'src,
) where
    SB: BitsInternal,
    DB: BitsInternal,
{
    match dst.format {
        PixelFormat::I420 => I420Writer::<DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader),
        ),
        PixelFormat::I422 => I422Writer::<DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI422::new(&dst.color, reader),
        ),
        PixelFormat::I444 => I444Writer::<DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI444::new(&src.color, reader),
        ),
        PixelFormat::NV12 => NV12Writer::<DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            RgbToI420::new(&dst.color, reader),
        ),
        PixelFormat::RGBA => RgbaWriter::<false, DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGRA => RgbaWriter::<true, DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::RGB => RgbWriter::<false, DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
        PixelFormat::BGR => RgbWriter::<true, DB, _>::read(
            dst.width,
            dst.height,
            dst.planes,
            dst.bits_per_component,
            dst.window,
            reader,
        ),
    }
}
