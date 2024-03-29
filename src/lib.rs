#![allow(clippy::missing_safety_doc)]

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

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    /// 3 Planes, Y, U, V
    ///
    /// 4x Y, 1x U, 1x V
    I420,

    // /// 4 Planes, Y, U, V, A
    // ///
    // /// 4x Y, 1x U, 1x V, 4x A
    // I420A,

    // /// 3 Planes, Y, U, V
    // ///
    // /// 4x Y, 2x U, 2x V
    // I422

    // /// 4 Planes, Y, U, V, A
    // ////
    // /// 4x Y, 2x U, 2x V, 4x A
    // I422A

    // /// 3 Planes, Y, U, V
    // ///
    // /// 1x Y, 1x U, 1x V
    // I444,

    // /// 2 Planes, Y, U & V interleaved
    // ///
    // /// 4x Y, 1x U & V
    // NV12,
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
            I420 => (width * height * 12).div_ceil(8),
            RGBA | BGRA => width * height * 4,
            RGB | BGR => width * height * 3,
        }
    }
}

macro_rules! write_i420 {
    ($dst:ident) => {
        RgbToI420Visitor::new(
            &$dst.color,
            I420Writer::<DB>::new(
                $dst.width,
                $dst.height,
                $dst.planes,
                $dst.bits_per_channel,
                $dst.window,
            ),
        )
    };
}

macro_rules! write_rgb {
    ($dst:ident) => {
        RGBWriter::<false, DB>::new(
            $dst.width,
            $dst.height,
            $dst.planes,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_bgr {
    ($dst:ident) => {
        RGBWriter::<true, DB>::new(
            $dst.width,
            $dst.height,
            $dst.planes,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_rgba {
    ($dst:ident) => {
        RGBAWriter::<false, DB>::new(
            $dst.width,
            $dst.height,
            $dst.planes,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_bgra {
    ($dst:ident) => {
        RGBAWriter::<true, DB>::new(
            $dst.width,
            $dst.height,
            $dst.planes,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! match_dst_format {
    ($src:ident, $dst:ident, $read_to_rgb:ident) => {
        match $dst.format {
            PixelFormat::I420 => $read_to_rgb!($src, $dst, write_i420!($dst)),
            PixelFormat::RGBA => $read_to_rgb!($src, $dst, write_rgba!($dst)),
            PixelFormat::BGRA => $read_to_rgb!($src, $dst, write_bgra!($dst)),
            PixelFormat::RGB => $read_to_rgb!($src, $dst, write_rgb!($dst)),
            PixelFormat::BGR => $read_to_rgb!($src, $dst, write_bgr!($dst)),
        }
    };
}

#[allow(private_bounds)]
pub fn convert<SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    verify_input_windows_same_size(&src, &dst);

    match src.format {
        PixelFormat::I420 => convert_i420::<SB, DB>(src, dst),
        PixelFormat::RGBA => convert_rgba::<false, SB, DB>(src, dst),
        PixelFormat::BGRA => convert_rgba::<true, SB, DB>(src, dst),
        PixelFormat::RGB => convert_rgb::<false, SB, DB>(src, dst),
        PixelFormat::BGR => convert_rgb::<true, SB, DB>(src, dst),
    }
}

fn convert_i420<SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    macro_rules! read_i420_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_i420::<SB, _>(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_channel,
                $src.window,
                I420ToRgbVisitor::new(
                    &$src.color,
                    RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
                ),
            )
        };
    }
    match_dst_format!(src, dst, read_i420_to_rgb);
}

fn convert_rgb<const REVERSE: bool, SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    macro_rules! read_rgb_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgb_4x::<REVERSE, SB, _>(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgb_to_rgb);
}

fn convert_rgba<const REVERSE: bool, SB, DB>(src: Source<'_, SB>, dst: Destination<'_, DB>)
where
    SB: BitsInternal,
    DB: BitsInternal,
{
    macro_rules! read_rgba_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgba_4x::<REVERSE, SB, _>(
                $src.width,
                $src.height,
                $src.planes,
                $src.bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgba_to_rgb);
}
