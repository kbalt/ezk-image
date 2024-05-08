use ezk_image::{
    convert_multi_thread, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, Image, PixelFormat,
    PixelFormatPlanes, Rect, ResizeAlg, Resizer,
};
use image::{Rgb, Rgba};

fn make_rgba8_image(primaries: ColorPrimaries, transfer: ColorTransfer) -> (Vec<u8>, usize, usize) {
    let primaries = primaries.xyz_to_rgb_mat();

    // Use odd image size to force non-simd paths
    let width = 2050;
    let height = 2050;

    let mut out = Vec::with_capacity(width * height * 4);

    let y = 0.5;

    for x in 0..height {
        let x = (x as f32) / 2050.0;

        for z in 0..width {
            let z = (z as f32) / 2050.0;

            let r = x * primaries[0][0] + y * primaries[1][0] + z * primaries[2][0];
            let g = x * primaries[0][1] + y * primaries[1][1] + z * primaries[2][1];
            let b = x * primaries[0][2] + y * primaries[1][2] + z * primaries[2][2];

            out.push((transfer.linear_to_scaled(r) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(g) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(b) * u8::MAX as f32) as u8);
            out.push(u8::MAX);
        }
    }

    (out, width, height)
}

fn make_i420_image(color: ColorInfo) -> (Vec<u8>, usize, usize) {
    let (rgba, width, height) = make_rgba8_image(color.primaries, color.transfer);
    let mut i420 = vec![255u8; PixelFormat::I420.buffer_size(width, height)];

    convert_multi_thread(
        Image::new(
            PixelFormat::RGBA,
            PixelFormatPlanes::RGBA(&rgba[..]),
            width,
            height,
            color,
            8,
        )
        .unwrap(),
        Image::new(
            PixelFormat::I420,
            PixelFormatPlanes::infer_i420(&mut i420[..], width, height),
            width,
            height,
            color,
            8,
        )
        .unwrap(),
    )
    .unwrap();

    (i420, width, height)
}

fn make_nv12_image(color: ColorInfo) -> (Vec<u8>, usize, usize) {
    let (rgba, width, height) = make_rgba8_image(color.primaries, color.transfer);
    let mut nv12 = vec![255u8; PixelFormat::NV12.buffer_size(width, height)];

    convert_multi_thread(
        Image::new(
            PixelFormat::RGBA,
            PixelFormatPlanes::RGBA(&rgba[..]),
            width,
            height,
            color,
            8,
        )
        .unwrap(),
        Image::new(
            PixelFormat::NV12,
            PixelFormatPlanes::infer_nv12(&mut nv12[..], width, height),
            width,
            height,
            color,
            8,
        )
        .unwrap(),
    )
    .unwrap();

    (nv12, width, height)
}

#[test]
fn i420_to_rgb() {
    let (i420, width, height) = make_i420_image(ColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    });

    let mut rgb = vec![0u8; PixelFormat::RGB.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::I420,
        PixelFormatPlanes::infer_i420(&i420[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb).unwrap();

    buffer.save("tests/I420_TO_RGB.png").unwrap();
}

#[test]
fn i420_to_rgba_with_window() {
    let (i420, width, height) = make_i420_image(ColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    });

    let mut rgb = vec![0u8; PixelFormat::RGB.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::I420,
        PixelFormatPlanes::infer_i420(&i420[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap()
    .with_window(Rect {
        x: 100,
        y: 200,
        width: 400,
        height: 400,
    })
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap()
    .with_window(Rect {
        x: 400,
        y: 700,
        width: 400,
        height: 400,
    })
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb).unwrap();

    buffer.save("tests/I420_TO_RGBA_WINDOW.png").unwrap();
}

#[test]
fn rgba_to_rgba() {
    let (rgba, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut rgba_dst = rgba.clone();
    rgba_dst.iter_mut().for_each(|b| *b = 255);

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba_dst[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba_dst)
            .unwrap();

    buffer.save("tests/RGBA_TO_RGBA.png").unwrap();
}

#[test]
fn rgba_to_rgb() {
    let (rgba, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::SRGB);

    let mut rgb_dst = vec![0u8; PixelFormat::RGB.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::SRGB,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb_dst[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb_dst).unwrap();

    buffer.save("tests/RGBA_TO_RGB.png").unwrap();
}

#[test]
fn i420_to_rgb_scale() {
    let (nv12, width, height) = make_nv12_image(ColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    });

    let scaled_width = 100;
    let scaled_height = 100;

    // First upscale NV12

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let mut nv12_upscaled = vec![0u8; PixelFormat::I420.buffer_size(scaled_width, scaled_height)];

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12_upscaled[..], scaled_width, scaled_height),
        scaled_width,
        scaled_height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    Resizer::new(ResizeAlg::Nearest).resize(src, dst).unwrap();

    // Convert I420 to RGB

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12_upscaled[..], scaled_width, scaled_height),
        scaled_width,
        scaled_height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let mut rgb = vec![0u8; PixelFormat::RGB.buffer_size(scaled_width, scaled_height)];

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb[..]),
        scaled_width,
        scaled_height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        scaled_width as _,
        scaled_height as _,
        rgb,
    )
    .unwrap();

    buffer.save("tests/NV12_UPSCALE.png").unwrap();
}

#[test]
fn rgba8_to_rgba16_and_back() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut rgb16 = vec![0u16; PixelFormat::RGB.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb16[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    {
        let buffer = image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_vec(
            width as _,
            height as _,
            rgb16.clone(),
        )
        .unwrap();

        buffer.save("tests/RGBA8_TO_RGB16.png").unwrap();
    }

    let src = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&rgb16[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/RGB16_TO_RGBA8.png").unwrap();
}

#[test]
fn rgba8_to_nv12_and_back() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = vec![0u8; PixelFormat::NV12.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/NV12_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_i422_and_back() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut i422 = vec![0u8; PixelFormat::I422.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::I422,
        PixelFormatPlanes::infer_i422(&mut i422[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::I422,
        PixelFormatPlanes::infer_i422(&i422[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/I422_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_i444_and_back() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut i444 = vec![0u8; PixelFormat::I444.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::I444,
        PixelFormatPlanes::infer_i444(&mut i444[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::I444,
        PixelFormatPlanes::infer_i444(&i444[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/I444_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_nv12_and_back_ictcp_pq() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = vec![0u16; PixelFormat::NV12.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::ICtCpPQ,
            transfer: ColorTransfer::BT2100PQ,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::ICtCpPQ,
            transfer: ColorTransfer::BT2100PQ,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/NV12_TO_RGBA_ICTCP_PQ.png").unwrap();
}

#[test]
fn rgba8_to_nv12_and_back_ictcp_hlg() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = vec![0u16; PixelFormat::NV12.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::ICtCpHLG,
            transfer: ColorTransfer::BT2100HLG,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12[..], width, height),
        width,
        height,
        ColorInfo {
            space: ColorSpace::ICtCpHLG,
            transfer: ColorTransfer::BT2100HLG,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        16,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        },
        8,
    )
    .unwrap();

    convert_multi_thread(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/NV12_TO_RGBA_ICTCP_HLG.png").unwrap();
}
