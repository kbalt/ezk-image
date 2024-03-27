#![allow(clippy::missing_safety_doc)]

use bits::{Bits, U16BE, U16LE, U8};
use formats::*;
use src_dst::RawMutSliceU8;

pub use color::{ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer};
#[cfg(feature = "multi-thread")]
pub use multi_thread::convert_multi_thread;
pub use src_dst::{Destination, Source};

mod bits;
mod color;
mod endian;
mod formats;
#[cfg(feature = "multi-thread")]
mod multi_thread;
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
fn verify_input(src: &Source<'_>, dst: &Destination<'_>) -> (Rect, Rect) {
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
    I4208,
    I42016LE,
    I42016BE,

    RGBA8,
    RGBA16LE,
    RGBA16BE,

    BGRA8,
    BGRA16LE,
    BGRA16BE,

    RGB8,
    RGB16LE,
    RGB16BE,

    BGR8,
    BGR16LE,
    BGR16BE,
    // I420A,
    // I422,
    // I444,
    // NV12,
}

impl PixelFormat {
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        use PixelFormat::*;

        match self {
            I4208 => (width * height * 12).div_ceil(8),
            I42016LE | I42016BE => (width * height * 24).div_ceil(8),
            RGBA8 | BGRA8 => width * height * 4,
            RGBA16LE | RGBA16BE | BGRA16LE | BGRA16BE => width * height * 8,
            RGB8 | BGR8 => width * height * 3,
            RGB16LE | RGB16BE | BGR16LE | BGR16BE => width * height * 6,
        }
    }
}

macro_rules! write_i420 {
    ($dst:ident, $primitive:ident) => {
        RgbToI420Visitor::new(
            &$dst.color,
            I420Writer::<$primitive>::new(
                $dst.width,
                $dst.height,
                $dst.buf,
                $dst.bits_per_channel,
                $dst.window,
            ),
        )
    };
}

macro_rules! write_rgb {
    ($dst:ident, $primitive:ident) => {
        RGBWriter::<false, $primitive>::new(
            $dst.width,
            $dst.height,
            $dst.buf,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_bgr {
    ($dst:ident, $primitive:ident) => {
        RGBWriter::<true, $primitive>::new(
            $dst.width,
            $dst.height,
            $dst.buf,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_rgba {
    ($dst:ident, $primitive:ident) => {
        RGBAWriter::<false, $primitive>::new(
            $dst.width,
            $dst.height,
            $dst.buf,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! write_bgra {
    ($dst:ident, $primitive:ident) => {
        RGBAWriter::<true, $primitive>::new(
            $dst.width,
            $dst.height,
            $dst.buf,
            $dst.bits_per_channel,
            $dst.window,
        )
    };
}

macro_rules! match_dst_format {
    ($src:ident, $dst:ident, $read_to_rgb:ident) => {
        match $dst.format {
            PixelFormat::I4208 => $read_to_rgb!($src, $dst, write_i420!($dst, U8)),
            PixelFormat::I42016LE => $read_to_rgb!($src, $dst, write_i420!($dst, U16LE)),
            PixelFormat::I42016BE => $read_to_rgb!($src, $dst, write_i420!($dst, U16BE)),

            PixelFormat::RGBA8 => $read_to_rgb!($src, $dst, write_rgba!($dst, U8)),
            PixelFormat::RGBA16LE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16LE)),
            PixelFormat::RGBA16BE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16BE)),

            PixelFormat::BGRA8 => $read_to_rgb!($src, $dst, write_bgra!($dst, U8)),
            PixelFormat::BGRA16LE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16LE)),
            PixelFormat::BGRA16BE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16BE)),

            PixelFormat::RGB8 => $read_to_rgb!($src, $dst, write_rgb!($dst, U8)),
            PixelFormat::RGB16LE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16LE)),
            PixelFormat::RGB16BE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16BE)),

            PixelFormat::BGR8 => $read_to_rgb!($src, $dst, write_bgr!($dst, U8)),
            PixelFormat::BGR16LE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16LE)),
            PixelFormat::BGR16BE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16BE)),
        }
    };
}

pub fn convert<'a>(src: Source<'a>, dst: Destination<'a>) {
    verify_input(&src, &dst);

    match src.format {
        PixelFormat::I4208 => convert_i420::<U8>(src, dst),
        PixelFormat::I42016LE => convert_i420::<U16LE>(src, dst),
        PixelFormat::I42016BE => convert_i420::<U16BE>(src, dst),

        PixelFormat::RGBA8 => convert_rgba::<false, U8>(src, dst),
        PixelFormat::RGBA16LE => convert_rgba::<false, U16LE>(src, dst),
        PixelFormat::RGBA16BE => convert_rgba::<false, U16BE>(src, dst),

        PixelFormat::BGRA8 => convert_rgba::<true, U8>(src, dst),
        PixelFormat::BGRA16LE => convert_rgba::<true, U16LE>(src, dst),
        PixelFormat::BGRA16BE => convert_rgba::<true, U16BE>(src, dst),

        PixelFormat::RGB8 => convert_rgb::<false, U8>(src, dst),
        PixelFormat::RGB16LE => convert_rgb::<false, U16LE>(src, dst),
        PixelFormat::RGB16BE => convert_rgb::<false, U16BE>(src, dst),

        PixelFormat::BGR8 => convert_rgb::<true, U8>(src, dst),
        PixelFormat::BGR16LE => convert_rgb::<true, U16LE>(src, dst),
        PixelFormat::BGR16BE => convert_rgb::<true, U16BE>(src, dst),
    }
}

fn convert_i420<'a, B: Bits>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_i420_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_i420::<B, _>(
                $src.width,
                $src.height,
                $src.buf,
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

fn convert_rgb<'a, const REVERSE: bool, B: Bits>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_rgb_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgb_4x::<REVERSE, B, _>(
                $src.width,
                $src.height,
                $src.buf,
                $src.bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgb_to_rgb);
}

fn convert_rgba<'a, const REVERSE: bool, B: Bits>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_rgba_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgba_4x::<REVERSE, B, _>(
                $src.width,
                $src.height,
                $src.buf,
                $src.bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgba_to_rgb);
}
