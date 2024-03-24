#![allow(clippy::missing_safety_doc)]

use endian::{BigEndian, Endian, LittleEndian};
use formats::*;
use src_dst::RawMutSliceU8;

pub use color::{ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer};
#[cfg(feature = "multi-thread")]
pub use multi_thread::convert_multi_thread;
pub use src_dst::{Destination, Source};

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

mod util {
    pub(crate) fn scale(depth: usize) -> f32 {
        ((1 << depth) - 1) as f32
    }
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

    // I420A,
    // I422,
    // I444,
    // NV12,
    RGB,
    RGBA,
    BGR,
    BGRA,
}

impl PixelFormat {
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        use PixelFormat::*;

        match self {
            I420 => (width * height * 12).div_ceil(8),
            I420P10LE | I420P10BE | I420P12LE | I420P12BE => (width * height * 24).div_ceil(8),
            RGB | BGR => width * height * 3,
            RGBA | BGRA => width * height * 4,
        }
    }
}

macro_rules! write_i420 {
    ($dst:ident) => {
        RgbToI420Visitor::new(
            &$dst.color,
            I420U8Writer::new($dst.width, $dst.height, $dst.buf, $dst.window),
        )
    };
}

macro_rules! write_i420_u16 {
    ($dst:ident, $bit_depth:expr, $endian:ident) => {
        RgbToI420Visitor::new(
            &$dst.color,
            I420U16Writer::<$bit_depth, $endian>::new(
                $dst.width,
                $dst.height,
                $dst.buf,
                $dst.window,
            ),
        )
    };
}

macro_rules! write_rgb {
    ($dst:ident) => {
        RGBWriter::<false>::new($dst.width, $dst.height, $dst.buf, $dst.window)
    };
}

macro_rules! write_rgba {
    ($dst:ident) => {
        RGBAWriter::<false>::new($dst.width, $dst.height, $dst.buf, $dst.window)
    };
}

macro_rules! write_bgr {
    ($dst:ident) => {
        RGBWriter::<true>::new($dst.width, $dst.height, $dst.buf, $dst.window)
    };
}

macro_rules! write_bgra {
    ($dst:ident) => {
        RGBAWriter::<true>::new($dst.width, $dst.height, $dst.buf, $dst.window)
    };
}

pub fn convert<'a>(src: Source<'a>, dst: Destination<'a>) {
    verify_input(&src, &dst);

    match src.format {
        PixelFormat::I420 => convert_i420_u8(src, dst),
        PixelFormat::I420P10LE => convert_i420_u16::<10, LittleEndian>(src, dst),
        PixelFormat::I420P10BE => convert_i420_u16::<10, BigEndian>(src, dst),
        PixelFormat::I420P12LE => convert_i420_u16::<12, LittleEndian>(src, dst),
        PixelFormat::I420P12BE => convert_i420_u16::<12, BigEndian>(src, dst),
        PixelFormat::RGB => convert_rgb::<false>(src, dst),
        PixelFormat::RGBA => convert_rgba::<false>(src, dst),
        PixelFormat::BGR => convert_rgb::<true>(src, dst),
        PixelFormat::BGRA => convert_rgba::<true>(src, dst),
    }
}

fn convert_i420_u8<'a>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_i420_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_i420(
                $src.width,
                $src.height,
                $src.buf,
                $src.window,
                I420ToRgbVisitor::new(
                    &$src.color,
                    RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
                ),
            )
        };
    }

    match dst.format {
        PixelFormat::I420 => read_i420_to_rgb!(src, dst, write_i420!(dst)),
        PixelFormat::I420P10LE => {
            read_i420_to_rgb!(src, dst, write_i420_u16!(dst, 10, LittleEndian))
        }
        PixelFormat::I420P10BE => {
            read_i420_to_rgb!(src, dst, write_i420_u16!(dst, 10, BigEndian))
        }
        PixelFormat::I420P12LE => {
            read_i420_to_rgb!(src, dst, write_i420_u16!(dst, 12, LittleEndian))
        }
        PixelFormat::I420P12BE => {
            read_i420_to_rgb!(src, dst, write_i420_u16!(dst, 12, BigEndian))
        }
        PixelFormat::RGB => read_i420_to_rgb!(src, dst, write_rgb!(dst)),
        PixelFormat::RGBA => read_i420_to_rgb!(src, dst, write_rgba!(dst)),
        PixelFormat::BGR => read_i420_to_rgb!(src, dst, write_bgr!(dst)),
        PixelFormat::BGRA => read_i420_to_rgb!(src, dst, write_bgra!(dst)),
    }
}

fn convert_i420_u16<'a, const BIT_DEPTH: usize, E: Endian>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_i420_u16_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_i420_u16::<BIT_DEPTH, E, _>(
                $src.width,
                $src.height,
                $src.buf,
                $src.window,
                I420ToRgbVisitor::new(
                    &$src.color,
                    RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
                ),
            )
        };
    }

    match dst.format {
        PixelFormat::I420 => read_i420_u16_to_rgb!(src, dst, write_i420!(dst)),
        PixelFormat::I420P10LE => {
            read_i420_u16_to_rgb!(src, dst, write_i420_u16!(dst, 10, LittleEndian))
        }
        PixelFormat::I420P10BE => {
            read_i420_u16_to_rgb!(src, dst, write_i420_u16!(dst, 10, BigEndian))
        }
        PixelFormat::I420P12LE => {
            read_i420_u16_to_rgb!(src, dst, write_i420_u16!(dst, 12, LittleEndian))
        }
        PixelFormat::I420P12BE => {
            read_i420_u16_to_rgb!(src, dst, write_i420_u16!(dst, 12, BigEndian))
        }
        PixelFormat::RGB => read_i420_u16_to_rgb!(src, dst, write_rgb!(dst)),
        PixelFormat::RGBA => read_i420_u16_to_rgb!(src, dst, write_rgba!(dst)),
        PixelFormat::BGR => read_i420_u16_to_rgb!(src, dst, write_bgr!(dst)),
        PixelFormat::BGRA => read_i420_u16_to_rgb!(src, dst, write_bgra!(dst)),
    }
}

fn convert_rgb<'a, const REVERSE: bool>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_rgb_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgb_4x::<REVERSE, _>(
                $src.width,
                $src.height,
                $src.buf,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match dst.format {
        PixelFormat::I420 => read_rgb_to_rgb!(src, dst, write_i420!(dst)),
        PixelFormat::I420P10LE => {
            read_rgb_to_rgb!(src, dst, write_i420_u16!(dst, 10, LittleEndian))
        }
        PixelFormat::I420P10BE => {
            read_rgb_to_rgb!(src, dst, write_i420_u16!(dst, 10, BigEndian))
        }
        PixelFormat::I420P12LE => {
            read_rgb_to_rgb!(src, dst, write_i420_u16!(dst, 12, LittleEndian))
        }
        PixelFormat::I420P12BE => {
            read_rgb_to_rgb!(src, dst, write_i420_u16!(dst, 12, BigEndian))
        }
        PixelFormat::RGB => read_rgb_to_rgb!(src, dst, write_rgb!(dst)),
        PixelFormat::RGBA => read_rgb_to_rgb!(src, dst, write_rgba!(dst)),
        PixelFormat::BGR => read_rgb_to_rgb!(src, dst, write_bgr!(dst)),
        PixelFormat::BGRA => read_rgb_to_rgb!(src, dst, write_bgra!(dst)),
    }
}

fn convert_rgba<'a, const REVERSE: bool>(src: Source<'a>, dst: Destination<'a>) {
    macro_rules! read_rgba_to_rgb {
        ($src:ident, $dst:ident, $writer:expr $(,)?) => {
            read_rgba_4x::<REVERSE, _>(
                $src.width,
                $src.height,
                $src.buf,
                $src.window,
                RgbTransferAndPrimariesConvert::new(&$src.color, &$dst.color, $writer),
            )
        };
    }

    match dst.format {
        PixelFormat::I420 => read_rgba_to_rgb!(src, dst, write_i420!(dst)),
        PixelFormat::I420P10LE => {
            read_rgba_to_rgb!(src, dst, write_i420_u16!(dst, 10, LittleEndian))
        }
        PixelFormat::I420P10BE => {
            read_rgba_to_rgb!(src, dst, write_i420_u16!(dst, 10, BigEndian))
        }
        PixelFormat::I420P12LE => {
            read_rgba_to_rgb!(src, dst, write_i420_u16!(dst, 12, LittleEndian))
        }
        PixelFormat::I420P12BE => {
            read_rgba_to_rgb!(src, dst, write_i420_u16!(dst, 12, BigEndian))
        }
        PixelFormat::RGB => read_rgba_to_rgb!(src, dst, write_rgb!(dst)),
        PixelFormat::RGBA => read_rgba_to_rgb!(src, dst, write_rgba!(dst)),
        PixelFormat::BGR => read_rgba_to_rgb!(src, dst, write_bgr!(dst)),
        PixelFormat::BGRA => read_rgba_to_rgb!(src, dst, write_bgra!(dst)),
    }
}
