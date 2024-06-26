use criterion::{criterion_group, criterion_main, Criterion};
#[cfg(feature = "multi-thread")]
use ezk_image::convert_multi_thread;
use ezk_image::{
    convert, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, Image, PixelFormat,
    PixelFormatPlanes, YuvColorInfo,
};
use std::hint::black_box;

const IMAGE_WIDTH: usize = 1920;
const IMAGE_HEIGHT: usize = 1080;

const NOOP_COLOR_INFO: ColorInfo = ColorInfo::YUV(YuvColorInfo {
    space: ColorSpace::BT709,
    transfer: ColorTransfer::Linear,
    primaries: ColorPrimaries::BT709,
    full_range: true,
});

fn do_convert(
    src_format: PixelFormat,
    src_planes: PixelFormatPlanes<&[u8]>,
    dst_format: PixelFormat,
    dst_planes: PixelFormatPlanes<&mut [u8]>,
) {
    let src = Image::new(
        src_format,
        src_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();
    let dst = Image::new(
        dst_format,
        dst_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();
}

#[cfg(feature = "multi-thread")]
fn do_convert_multi_thread(
    src_format: PixelFormat,
    src_planes: PixelFormatPlanes<&[u8]>,
    dst_format: PixelFormat,
    dst_planes: PixelFormatPlanes<&mut [u8]>,
) {
    let src = Image::new(
        src_format,
        src_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();
    let dst = Image::new(
        dst_format,
        dst_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();
}

type ConvertFunction =
    fn(PixelFormat, PixelFormatPlanes<&[u8]>, PixelFormat, PixelFormatPlanes<&mut [u8]>);

fn run_benchmarks(c: &mut Criterion, do_convert: ConvertFunction, s: &str) {
    use PixelFormat::*;

    let mut rgb = black_box(vec![0u8; RGB.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);
    let mut rgba = black_box(vec![0u8; RGBA.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);
    let mut i420 = black_box(vec![0u8; I420.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);

    c.bench_function(&format!("RGB to I420 {s}"), |b| {
        b.iter(|| {
            do_convert(
                RGB,
                PixelFormatPlanes::RGB(&rgb),
                I420,
                PixelFormatPlanes::infer_i420(&mut i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
            )
        })
    });

    c.bench_function(&format!("I420 to RGB {s}"), |b| {
        b.iter(|| {
            do_convert(
                I420,
                PixelFormatPlanes::infer_i420(&i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
                RGB,
                PixelFormatPlanes::RGB(&mut rgb),
            );
        })
    });

    c.bench_function(&format!("RGBA to I420 {s}"), |b| {
        b.iter(|| {
            do_convert(
                RGBA,
                PixelFormatPlanes::RGBA(&rgba),
                I420,
                PixelFormatPlanes::infer_i420(&mut i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
            )
        })
    });

    c.bench_function(&format!("I420 to RGBA {s}"), |b| {
        b.iter(|| {
            do_convert(
                I420,
                PixelFormatPlanes::infer_i420(&i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
                RGBA,
                PixelFormatPlanes::RGBA(&mut rgba),
            );
        })
    });

    c.bench_function(&format!("RGBA to NV12 {s}"), |b| {
        b.iter(|| {
            do_convert(
                RGBA,
                PixelFormatPlanes::RGBA(&rgba),
                NV12,
                PixelFormatPlanes::infer_nv12(&mut i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
            )
        })
    });

    c.bench_function(&format!("NV12 to RGBA {s}"), |b| {
        b.iter(|| {
            do_convert(
                NV12,
                PixelFormatPlanes::infer_nv12(&i420[..], IMAGE_WIDTH, IMAGE_HEIGHT),
                RGBA,
                PixelFormatPlanes::RGBA(&mut rgba),
            );
        })
    });
}

fn single_threaded(c: &mut Criterion) {
    run_benchmarks(c, do_convert, "single threaded")
}

#[cfg(feature = "multi-thread")]
fn multi_threaded(c: &mut Criterion) {
    run_benchmarks(c, do_convert_multi_thread, "multi threaded")
}

#[cfg(feature = "multi-thread")]
criterion_group!(img, single_threaded, multi_threaded);

#[cfg(not(feature = "multi-thread"))]
criterion_group!(img, single_threaded);

criterion_main!(img);
