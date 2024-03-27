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
    I420,
    I420P10LE,
    I420P10BE,
    I420P12LE,
    I420P12BE,

    RGBA,
    RGBAP10LE,
    RGBAP10BE,
    RGBAP12LE,
    RGBAP12BE,

    BGRA,
    BGRAP10LE,
    BGRAP10BE,
    BGRAP12LE,
    BGRAP12BE,

    RGB,
    RGBP10LE,
    RGBP10BE,
    RGBP12LE,
    RGBP12BE,

    BGR,
    BGRP10LE,
    BGRP10BE,
    BGRP12LE,
    BGRP12BE,
    // I420A,
    // I422,
    // I444,
    // NV12,
}

impl PixelFormat {
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        use PixelFormat::*;

        match self {
            I420 => (width * height * 12).div_ceil(8),
            I420P10LE | I420P10BE | I420P12LE | I420P12BE => (width * height * 24).div_ceil(8),
            RGBA | BGRA => width * height * 4,
            RGBAP10LE | RGBAP10BE | RGBAP12LE | RGBAP12BE | BGRAP10LE | BGRAP10BE | BGRAP12LE
            | BGRAP12BE => width * height * 8,
            RGB | BGR => width * height * 3,
            RGBP10LE | RGBP10BE | RGBP12LE | RGBP12BE | BGRP10LE | BGRP10BE | BGRP12LE
            | BGRP12BE => width * height * 6,
        }
    }
}

macro_rules! write_i420 {
    ($dst:ident, $primitive:ident, $bits:expr) => {
        RgbToI420Visitor::new(
            &$dst.color,
            I420Writer::<$primitive>::new($dst.width, $dst.height, $dst.buf, $bits, $dst.window),
        )
    };
}

macro_rules! write_rgb {
    ($dst:ident, $primitive:ident, $bits:expr) => {
        RGBWriter::<false, $primitive>::new($dst.width, $dst.height, $dst.buf, $bits, $dst.window)
    };
}

macro_rules! write_bgr {
    ($dst:ident, $primitive:ident, $bits:expr) => {
        RGBWriter::<true, $primitive>::new($dst.width, $dst.height, $dst.buf, $bits, $dst.window)
    };
}

macro_rules! write_rgba {
    ($dst:ident, $primitive:ident, $bits:expr) => {
        RGBAWriter::<false, $primitive>::new($dst.width, $dst.height, $dst.buf, $bits, $dst.window)
    };
}

macro_rules! write_bgra {
    ($dst:ident, $primitive:ident, $bits:expr) => {
        RGBAWriter::<true, $primitive>::new($dst.width, $dst.height, $dst.buf, $bits, $dst.window)
    };
}

macro_rules! match_dst_format {
    ($src:ident, $dst:ident, $read_to_rgb:ident) => {
        match $dst.format {
            PixelFormat::I420 => $read_to_rgb!($src, $dst, write_i420!($dst, U8, 8)),
            PixelFormat::I420P10LE => $read_to_rgb!($src, $dst, write_i420!($dst, U16LE, 10)),
            PixelFormat::I420P10BE => $read_to_rgb!($src, $dst, write_i420!($dst, U16BE, 10)),
            PixelFormat::I420P12LE => $read_to_rgb!($src, $dst, write_i420!($dst, U16LE, 12)),
            PixelFormat::I420P12BE => $read_to_rgb!($src, $dst, write_i420!($dst, U16BE, 12)),

            PixelFormat::RGBA => $read_to_rgb!($src, $dst, write_rgba!($dst, U8, 8)),
            PixelFormat::RGBAP10LE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16LE, 10)),
            PixelFormat::RGBAP10BE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16BE, 10)),
            PixelFormat::RGBAP12LE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16LE, 12)),
            PixelFormat::RGBAP12BE => $read_to_rgb!($src, $dst, write_rgba!($dst, U16BE, 12)),

            PixelFormat::BGRA => $read_to_rgb!($src, $dst, write_bgra!($dst, U8, 8)),
            PixelFormat::BGRAP10LE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16LE, 10)),
            PixelFormat::BGRAP10BE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16BE, 10)),
            PixelFormat::BGRAP12LE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16LE, 12)),
            PixelFormat::BGRAP12BE => $read_to_rgb!($src, $dst, write_bgra!($dst, U16BE, 12)),

            PixelFormat::RGB => $read_to_rgb!($src, $dst, write_rgb!($dst, U8, 8)),
            PixelFormat::RGBP10LE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16LE, 10)),
            PixelFormat::RGBP10BE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16BE, 10)),
            PixelFormat::RGBP12LE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16LE, 12)),
            PixelFormat::RGBP12BE => $read_to_rgb!($src, $dst, write_rgb!($dst, U16BE, 12)),

            PixelFormat::BGR => $read_to_rgb!($src, $dst, write_bgr!($dst, U8, 8)),
            PixelFormat::BGRP10LE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16LE, 10)),
            PixelFormat::BGRP10BE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16BE, 10)),
            PixelFormat::BGRP12LE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16LE, 12)),
            PixelFormat::BGRP12BE => $read_to_rgb!($src, $dst, write_bgr!($dst, U16BE, 12)),
        }
    };
}

pub fn convert<'a>(src: Source<'a>, dst: Destination<'a>) {
    verify_input(&src, &dst);

    match src.format {
        PixelFormat::I420 => convert_i420::<U8>(src, dst, 8),
        PixelFormat::I420P10LE => convert_i420::<U16LE>(src, dst, 10),
        PixelFormat::I420P10BE => convert_i420::<U16BE>(src, dst, 10),
        PixelFormat::I420P12LE => convert_i420::<U16LE>(src, dst, 12),
        PixelFormat::I420P12BE => convert_i420::<U16BE>(src, dst, 12),

        PixelFormat::RGBA => convert_rgba::<false, U8>(src, dst, 8),
        PixelFormat::RGBAP10LE => convert_rgba::<false, U16LE>(src, dst, 10),
        PixelFormat::RGBAP10BE => convert_rgba::<false, U16BE>(src, dst, 10),
        PixelFormat::RGBAP12LE => convert_rgba::<false, U16LE>(src, dst, 12),
        PixelFormat::RGBAP12BE => convert_rgba::<false, U16BE>(src, dst, 12),

        PixelFormat::BGRA => convert_rgba::<true, U8>(src, dst, 8),
        PixelFormat::BGRAP10LE => convert_rgba::<true, U16LE>(src, dst, 10),
        PixelFormat::BGRAP10BE => convert_rgba::<true, U16BE>(src, dst, 10),
        PixelFormat::BGRAP12LE => convert_rgba::<true, U16LE>(src, dst, 12),
        PixelFormat::BGRAP12BE => convert_rgba::<true, U16BE>(src, dst, 12),

        PixelFormat::RGB => convert_rgb::<false, U8>(src, dst, 8),
        PixelFormat::RGBP10LE => convert_rgb::<false, U16LE>(src, dst, 10),
        PixelFormat::RGBP10BE => convert_rgb::<false, U16BE>(src, dst, 10),
        PixelFormat::RGBP12LE => convert_rgb::<false, U16LE>(src, dst, 12),
        PixelFormat::RGBP12BE => convert_rgb::<false, U16BE>(src, dst, 12),

        PixelFormat::BGR => convert_rgb::<true, U8>(src, dst, 8),
        PixelFormat::BGRP10LE => convert_rgb::<true, U16LE>(src, dst, 10),
        PixelFormat::BGRP10BE => convert_rgb::<true, U16BE>(src, dst, 10),
        PixelFormat::BGRP12LE => convert_rgb::<true, U16LE>(src, dst, 12),
        PixelFormat::BGRP12BE => convert_rgb::<true, U16BE>(src, dst, 12),
    }
}

fn convert_i420<'a, B: Bits>(src: Source<'a>, dst: Destination<'a>, src_bits_per_channel: usize) {
    macro_rules! read_i420_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_i420::<B, _>(
                $src.width,
                $src.height,
                $src.buf,
                src_bits_per_channel,
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

fn convert_rgb<'a, const REVERSE: bool, B: Bits>(
    src: Source<'a>,
    dst: Destination<'a>,
    src_bits_per_channel: usize,
) {
    macro_rules! read_rgb_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgb_4x::<REVERSE, _>(
                $src.width,
                $src.height,
                $src.buf,
                src_bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgb_to_rgb);
}

fn convert_rgba<'a, const REVERSE: bool, B: Bits>(
    src: Source<'a>,
    dst: Destination<'a>,
    src_bits_per_channel: usize,
) {
    macro_rules! read_rgba_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgba_4x::<REVERSE, B, _>(
                $src.width,
                $src.height,
                $src.buf,
                src_bits_per_channel,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match_dst_format!(src, dst, read_rgba_to_rgb);
}
