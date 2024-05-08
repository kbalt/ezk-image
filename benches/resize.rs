use criterion::{criterion_group, criterion_main, Criterion};
use ezk_image::{
    ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, Image, PixelFormat, PixelFormatPlanes,
    Resizer,
};
use fir::ResizeAlg;
use std::hint::black_box;

const IMAGE_DIM_LO: (usize, usize) = (1280, 720);
const IMAGE_DIM_HI: (usize, usize) = (1920, 1080);

const NOOP_COLOR_INFO: ColorInfo = ColorInfo {
    space: ColorSpace::BT709,
    transfer: ColorTransfer::Linear,
    primaries: ColorPrimaries::BT709,
    full_range: true,
};

fn do_resize(
    format: PixelFormat,
    src_buf: &[u8],
    dst_buf: &mut [u8],

    src_dim: (usize, usize),
    dst_dim: (usize, usize),
) {
    let src = Image::new(
        format,
        PixelFormatPlanes::infer(format, src_buf, src_dim.0, src_dim.1),
        src_dim.0,
        src_dim.1,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();

    let dst = Image::new(
        format,
        PixelFormatPlanes::infer(format, dst_buf, dst_dim.0, dst_dim.1),
        dst_dim.0,
        dst_dim.1,
        NOOP_COLOR_INFO,
        8,
    )
    .unwrap();

    Resizer::new(ResizeAlg::Convolution(fir::FilterType::Bilinear))
        .resize(src, dst)
        .unwrap();
}

fn bench_format(c: &mut Criterion, format: PixelFormat) {
    let mut buf_lo = black_box(vec![
        0u8;
        format.buffer_size(IMAGE_DIM_LO.0, IMAGE_DIM_LO.1)
    ]);
    let mut buf_hi = black_box(vec![
        0u8;
        format.buffer_size(IMAGE_DIM_HI.0, IMAGE_DIM_HI.1)
    ]);

    c.bench_function(&format!("{format:?} upscale"), |b| {
        b.iter(|| do_resize(format, &buf_lo, &mut buf_hi, IMAGE_DIM_LO, IMAGE_DIM_HI))
    });

    c.bench_function(&format!("{format:?} downscale"), |b| {
        b.iter(|| do_resize(format, &buf_hi, &mut buf_lo, IMAGE_DIM_HI, IMAGE_DIM_LO))
    });
}

fn resize(c: &mut Criterion) {
    bench_format(c, PixelFormat::I420);
    bench_format(c, PixelFormat::I422);
    bench_format(c, PixelFormat::I444);
    bench_format(c, PixelFormat::NV12);
    bench_format(c, PixelFormat::RGB);
    bench_format(c, PixelFormat::RGBA);
}

criterion_group!(img, resize);
criterion_main!(img);
