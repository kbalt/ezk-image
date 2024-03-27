use ezk_image::{
    convert_multi_thread, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer, Destination,
    PixelFormat, Source,
};

use image::{Rgb, Rgba};

fn make_rgba8_image(primaries: ColorPrimaries, transfer: ColorTransfer) -> (Vec<u8>, usize, usize) {
    let primaries = primaries.xyz_to_rgb_mat();

    // Use odd image size to force non-simd paths
    let width = 4098;
    let height = 4098;

    let mut out = Vec::with_capacity(width * height * 4);

    let y = 0.5;

    for x in 0..height {
        let x = (x as f32) / 4098.0;

        for z in 0..width {
            let z = 1.0 - ((z as f32) / 4098.0);

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

    let mut i420 = vec![0u8; PixelFormat::I4208.buffer_size(width, height)];

    convert_multi_thread(
        Source::new(PixelFormat::RGBA8, color, &rgba, width, height, 8),
        Destination::new(PixelFormat::I4208, color, &mut i420, width, height, 8),
    );

    (i420, width, height)
}

#[test]
fn i420_to_rgba() {
    let (i420, width, height) = make_i420_image(ColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    });

    let mut rgb = vec![0u8; PixelFormat::RGBA8.buffer_size(width, height)];

    let src = Source::new(
        PixelFormat::I4208,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &i420,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT2100HLG,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgb,
        width,
        height,
        8,
    );

    crate::convert_multi_thread(src, dst);

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb).unwrap();

    buffer.save("tests/I420_TO_RGBA.png").unwrap();
}

#[test]
fn rgba_to_rgba() {
    let (rgba, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::SRGB);

    let mut rgba_dst = rgba.clone();

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::SRGB,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba_dst,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba_dst)
            .unwrap();

    buffer.save("tests/RGBA_TO_RGBA.png").unwrap();
}

#[test]
fn rgba_to_rgb() {
    let (rgba, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::SRGB);

    let mut rgb_dst = rgba.clone();

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::SRGB,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGB8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgb_dst,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb_dst).unwrap();

    buffer.save("tests/RGBA_TO_RGB.png").unwrap();
}

#[test]
fn rgba_to_i420_10_le() {
    let (mut rgba, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::Linear);

    let mut i420_dst = vec![0u8; PixelFormat::I42016LE.buffer_size(width, height)];

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::I42016LE,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut i420_dst,
        width,
        height,
        10,
    );

    convert_multi_thread(src, dst);

    let src = Source::new(
        PixelFormat::I42016LE,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &i420_dst,
        width,
        height,
        10,
    );

    let dst = Destination::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba).unwrap();

    buffer.save("tests/RGBA_TO_I420P10LE.png").unwrap();
}

#[test]
fn rgb_to_rgba() {
    let (mut rgba, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::Linear);

    let mut rgb_dst = vec![0u8; PixelFormat::RGB8.buffer_size(width, height)];

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGB8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgb_dst,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let src = Source::new(
        PixelFormat::RGB8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgb_dst,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb_dst).unwrap();

    buffer.save("tests/RGB_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_rgba16() {
    let (rgba8, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::Linear);

    let mut rgba16 = vec![0u8; PixelFormat::RGB16LE.buffer_size(width, height)];

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba8,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGB16LE,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba16,
        width,
        height,
        16,
    );

    convert_multi_thread(src, dst);

    let rgba16: Vec<u16> = rgba16
        .chunks_exact(2)
        .map(|b| u16::from_be_bytes([b[0], b[1]]))
        .collect();

    let buffer =
        image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_vec(width as _, height as _, rgba16)
            .unwrap();

    buffer.save("tests/RGBA8_TO_RGB16.png").unwrap();
}

#[test]
fn rgba16_to_rgba8() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::SRGB, ColorTransfer::Linear);

    let mut rgba16 = vec![0u8; PixelFormat::RGBA16LE.buffer_size(width, height)];

    let src = Source::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba8,
        width,
        height,
        8,
    );

    let dst = Destination::new(
        PixelFormat::RGBA16LE,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba16,
        width,
        height,
        16,
    );

    convert_multi_thread(src, dst);

    let src = Source::new(
        PixelFormat::RGBA16LE,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &rgba16,
        width,
        height,
        16,
    );

    let dst = Destination::new(
        PixelFormat::RGBA8,
        ColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::SRGB,
            full_range: false,
        },
        &mut rgba8,
        width,
        height,
        8,
    );

    convert_multi_thread(src, dst);

    let rgb_dst: Vec<u16> = rgba16
        .chunks_exact(2)
        .map(|b| u16::from_be_bytes([b[0], b[1]]))
        .collect();

    let buffer =
        image::ImageBuffer::<Rgba<u16>, Vec<u16>>::from_vec(width as _, height as _, rgb_dst)
            .unwrap();

    buffer.save("tests/RGBA16_TO_RGBA8.png").unwrap();
}
