use criterion::{criterion_group, criterion_main, Criterion};
use ezk_image::{
    convert, convert_multi_thread, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer,
    Destination, PixelFormat, Source,
};
use std::hint::black_box;

const IMAGE_WIDTH: usize = 1026;
const IMAGE_HEIGHT: usize = 1026;

const NOOP_COLOR_INFO: ColorInfo = ColorInfo {
    space: ColorSpace::BT709,
    transfer: ColorTransfer::Linear,
    primaries: ColorPrimaries::SRGB,
    full_range: true,
};

fn do_convert(src_format: PixelFormat, src: &[u8], dst_format: PixelFormat, dst: &mut [u8]) {
    let src = Source::new(src_format, NOOP_COLOR_INFO, src, IMAGE_WIDTH, IMAGE_HEIGHT);
    let dst = Destination::new(dst_format, NOOP_COLOR_INFO, dst, IMAGE_WIDTH, IMAGE_HEIGHT);

    convert(src, dst);
}

fn do_convert_multi_thread(
    src_format: PixelFormat,
    src: &[u8],
    dst_format: PixelFormat,
    dst: &mut [u8],
) {
    let src = Source::new(src_format, NOOP_COLOR_INFO, src, IMAGE_WIDTH, IMAGE_HEIGHT);
    let dst = Destination::new(dst_format, NOOP_COLOR_INFO, dst, IMAGE_WIDTH, IMAGE_HEIGHT);

    convert_multi_thread(src, dst);
}

fn run_benchmarks(
    c: &mut Criterion,
    do_convert: fn(PixelFormat, &[u8], PixelFormat, &mut [u8]),
    s: &str,
) {
    use PixelFormat::*;

    let mut rgb = black_box(vec![0u8; RGB.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);
    let mut rgba = black_box(vec![0u8; RGBA.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);
    let mut i420 = black_box(vec![0u8; I420.buffer_size(IMAGE_WIDTH, IMAGE_HEIGHT)]);

    c.bench_function(&format!("RGB to I420 {s}"), |b| {
        b.iter(|| do_convert(RGB, &rgb, I420, &mut i420))
    });

    c.bench_function(&format!("I420 to RGB {s}"), |b| {
        b.iter(|| do_convert(I420, &i420, RGB, &mut rgb))
    });

    c.bench_function(&format!("RGBA to I420 {s}"), |b| {
        b.iter(|| do_convert(RGBA, &rgba, I420, &mut i420))
    });

    c.bench_function(&format!("I420 to RGBA {s}"), |b| {
        b.iter(|| do_convert(I420, &i420, RGBA, &mut rgba))
    });
}

fn single_threaded(c: &mut Criterion) {
    run_benchmarks(c, do_convert, "single threaded")
}

fn multi_threaded(c: &mut Criterion) {
    run_benchmarks(c, do_convert_multi_thread, "multi threaded")
}

criterion_group!(img, single_threaded, multi_threaded);
criterion_main!(img);
