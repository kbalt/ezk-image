use criterion::{criterion_group, criterion_main, Criterion};
use ezk_image::{
    convert, convert_multi_thread, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer,
    Destination, PixelFormat, PixelFormatPlanes, Source, U8,
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

fn do_convert(
    src_format: PixelFormat,
    src_planes: PixelFormatPlanes<&[u8]>,
    dst_format: PixelFormat,
    dst_planes: PixelFormatPlanes<&mut [u8]>,
) {
    let src = Source::<U8>::new(
        src_format,
        src_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    );
    let dst = Destination::<U8>::new(
        dst_format,
        dst_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    );

    convert(src, dst);
}

fn do_convert_multi_thread(
    src_format: PixelFormat,
    src_planes: PixelFormatPlanes<&[u8]>,
    dst_format: PixelFormat,
    dst_planes: PixelFormatPlanes<&mut [u8]>,
) {
    let src = Source::<U8>::new(
        src_format,
        src_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    );
    let dst = Destination::<U8>::new(
        dst_format,
        dst_planes,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NOOP_COLOR_INFO,
        8,
    );

    convert_multi_thread(src, dst);
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

fn multi_threaded(c: &mut Criterion) {
    run_benchmarks(c, do_convert_multi_thread, "multi threaded")
}

criterion_group!(img, single_threaded, multi_threaded);
criterion_main!(img);
