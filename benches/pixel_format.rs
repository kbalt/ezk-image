use criterion::{criterion_group, criterion_main, Criterion};

use ezk_image::{
    convert, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, Image, PixelFormat, YuvColorInfo,
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

fn do_convert(src: &Image<Vec<u8>>, dst: &mut Image<Vec<u8>>) {
    convert(black_box(src), black_box(dst)).unwrap();
}

#[cfg(feature = "multi-thread")]
fn do_convert_multi_thread(src: &Image<Vec<u8>>, dst: &mut Image<Vec<u8>>) {
    use ezk_image::convert_multi_thread;

    convert_multi_thread(src, dst).unwrap();
}

type ConvertFunction = fn(&Image<Vec<u8>>, &mut Image<Vec<u8>>);

fn run_benchmarks(c: &mut Criterion, do_convert: ConvertFunction, s: &str) {
    use PixelFormat::*;

    let mut rgb = Image::blank(RGB, IMAGE_WIDTH, IMAGE_HEIGHT, NOOP_COLOR_INFO);
    let mut rgba = Image::blank(RGBA, IMAGE_WIDTH, IMAGE_HEIGHT, NOOP_COLOR_INFO);
    let mut i420 = Image::blank(I420, IMAGE_WIDTH, IMAGE_HEIGHT, NOOP_COLOR_INFO);
    let mut nv12 = Image::blank(NV12, IMAGE_WIDTH, IMAGE_HEIGHT, NOOP_COLOR_INFO);
    let mut i012 = Image::blank(I012, IMAGE_WIDTH, IMAGE_HEIGHT, NOOP_COLOR_INFO);

    c.bench_function(&format!("RGB to I420 {s}"), |b| {
        b.iter(|| {
            do_convert(&rgb, &mut i420);
        })
    });

    c.bench_function(&format!("I420 to RGB {s}"), |b| {
        b.iter(|| {
            do_convert(&i420, &mut rgb);
        })
    });

    c.bench_function(&format!("RGBA to I420 {s}"), |b| {
        b.iter(|| {
            do_convert(&rgba, &mut i420);
        })
    });

    c.bench_function(&format!("I420 to RGBA {s}"), |b| {
        b.iter(|| {
            do_convert(&i420, &mut rgba);
        })
    });

    c.bench_function(&format!("RGBA to NV12 {s}"), |b| {
        b.iter(|| do_convert(&rgba, &mut nv12))
    });

    c.bench_function(&format!("NV12 to RGBA {s}"), |b| {
        b.iter(|| {
            do_convert(&nv12, &mut rgba);
        })
    });

    c.bench_function(&format!("RGBA to I012 {s}"), |b| {
        b.iter(|| {
            do_convert(&rgba, &mut i012);
        })
    });

    c.bench_function(&format!("I012 to RGBA {s}"), |b| {
        b.iter(|| {
            do_convert(&i012, &mut rgba);
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
