use ezk_image::{
    convert, infer_i420, infer_i422, infer_nv12, ColorInfo, ColorPrimaries, ColorSpace,
    ColorTransfer, Image, ImageMut, ImageRef, ImageRefExt, PixelFormat, RgbColorInfo, Window,
    YuvColorInfo,
};
use image::{Rgb, Rgba};

fn make_rgba8_image(primaries: ColorPrimaries, transfer: ColorTransfer) -> Image<Vec<u8>> {
    let mat = primaries.xyz_to_rgb_mat();

    // Use odd image size to force non-simd paths
    let width = 640;
    let height = 360;

    let mut out = Vec::with_capacity(width * height * 4);

    let y = 0.4;

    for x in 0..height {
        let x = (x as f32) / 640.0;

        for z in 0..width {
            let z = (z as f32) / 360.0;

            let r = x * mat[0][0] + y * mat[1][0] + z * mat[2][0];
            let g = x * mat[0][1] + y * mat[1][1] + z * mat[2][1];
            let b = x * mat[0][2] + y * mat[1][2] + z * mat[2][2];

            out.push((transfer.linear_to_scaled(r) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(g) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(b) * u8::MAX as f32) as u8);
            out.push(u8::MAX);
        }
    }

    Image::new(
        PixelFormat::RGBA,
        vec![out],
        vec![width * 4],
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer,
            primaries,
        }),
    )
    .unwrap()
}

fn make_i420_image(color: YuvColorInfo) -> Image<Vec<u8>> {
    let rgba = make_rgba8_image(color.primaries, color.transfer);

    let mut i420 = Image::blank(
        PixelFormat::I420,
        rgba.width(),
        rgba.height(),
        ColorInfo::YUV(color),
    );

    convert(&rgba, &mut i420).unwrap();

    i420
}

fn make_nv12_image(color: YuvColorInfo) -> Image<Vec<u8>> {
    let rgba = make_rgba8_image(color.primaries, color.transfer);

    let mut nv12 = Image::blank(
        PixelFormat::NV12,
        rgba.width(),
        rgba.height(),
        ColorInfo::YUV(color),
    );

    convert(&rgba, &mut nv12).unwrap();

    nv12
}

#[test]
fn i420_to_rgb() {
    let i420 = make_i420_image(YuvColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    });

    let mut rgb = Image::blank(
        PixelFormat::RGB,
        i420.width(),
        i420.height(),
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
    );

    convert(&i420, &mut rgb).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        i420.width() as _,
        i420.height() as _,
        rgb.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/I420_TO_RGB.png").unwrap();
}

#[test]
fn i420_to_rgb_window() {
    let i420 = make_i420_image(YuvColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: true,
    })
    .crop(Window {
        x: 100,
        y: 50,
        width: 300,
        height: 300,
    })
    .unwrap();

    let mut rgb = Image::blank(
        PixelFormat::RGB,
        640,
        360,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
    )
    .crop(Window {
        x: 125,
        y: 10,
        width: i420.width(),
        height: i420.height(),
    })
    .unwrap();

    convert(&i420, &mut rgb).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        640,
        320,
        rgb.into_inner().planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/I420_TO_RGB_WINDOW.png").unwrap();
}
/*
#[test]
fn i420_to_rgba_with_window() {
    let (i420, width, height) = make_i420_image(YuvColorInfo {
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
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        8,
    )
    .unwrap()
    .with_window(Window {
        x: 100,
        y: 100,
        width: 200,
        height: 200,
    })
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap()
    .with_window(Window {
        x: 100,
        y: 100,
        width: 200,
        height: 200,
    })
    .unwrap();

    convert(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width as _, height as _, rgb).unwrap();

    buffer.save("tests/I420_TO_RGBA_WINDOW.png").unwrap();
}
*/
#[test]
fn rgba_to_rgba() {
    let rgba1 = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut rgba2 = rgba1.clone();
    rgba2
        .planes_mut()
        .next()
        .unwrap()
        .0
        .iter_mut()
        .for_each(|b| *b = 255);

    convert(&rgba1, &mut rgba2).unwrap();

    let buffer = image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
        rgba2.width() as _,
        rgba2.height() as _,
        rgba2.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/RGBA_TO_RGBA.png").unwrap();
}

#[test]
fn rgba_to_rgb() {
    let rgba = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::SRGB);

    let mut rgb_dst = vec![0u8; PixelFormat::RGB.buffer_size(rgba.width(), rgba.height())];

    let mut rgb = Image::new(
        PixelFormat::RGB,
        vec![&mut rgb_dst[..]],
        vec![rgba.width() * 3],
        rgba.width(),
        rgba.height(),
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
    )
    .unwrap();

    convert(&rgba, &mut rgb).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        rgb.width() as _,
        rgb.height() as _,
        rgb_dst,
    )
    .unwrap();

    buffer.save("tests/RGBA_TO_RGB.png").unwrap();
}
/*
#[test]
fn i420_to_rgb_scale() {
    let (nv12, width, height) = make_nv12_image(YuvColorInfo {
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
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        8,
    )
    .unwrap();

    let mut nv12_upscaled = vec![0u8; PixelFormat::I420.buffer_size(scaled_width, scaled_height)];

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12_upscaled[..], scaled_width, scaled_height),
        scaled_width,
        scaled_height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
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
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        8,
    )
    .unwrap();

    let mut rgb = vec![0u8; PixelFormat::RGB.buffer_size(scaled_width, scaled_height)];

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb[..]),
        scaled_width,
        scaled_height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

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

    let mut rgb16 = vec![0u8; PixelFormat::RGB.buffer_size(width, height) * 2];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb16[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        16,
    )
    .unwrap();

    convert(src, dst).unwrap();

    {
        // let buffer = image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_vec(
        //     width as _,
        //     height as _,
        //     rgb16.clone(),
        // )
        // .unwrap();

        // buffer.save("tests/RGBA8_TO_RGB16.png").unwrap();
    }

    let src = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&rgb16[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        16,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/RGB16_TO_RGBA8.png").unwrap();
}
*/
#[test]
fn rgba8_to_nv12_and_back() {
    let mut rgba = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = Image::blank(
        PixelFormat::NV12,
        rgba.width(),
        rgba.height(),
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
    );

    convert(&rgba, &mut nv12).unwrap();

    rgba.planes_mut().next().unwrap().0.fill(255);

    convert(&nv12, &mut rgba).unwrap();

    let buffer = image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
        rgba.width() as _,
        rgba.height() as _,
        rgba.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/NV12_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_i422_and_back() {
    let mut rgba = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut i422 = Image::blank(
        PixelFormat::I422,
        rgba.width(),
        rgba.height(),
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
    );

    convert(&rgba, &mut i422).unwrap();

    convert(&i422, &mut rgba).unwrap();

    let buffer = image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(
        rgba.width() as _,
        rgba.height() as _,
        rgba.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/I422_TO_RGBA.png").unwrap();
}

/*

#[test]
fn rgba8_to_i444_and_back() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut i444 = vec![0u8; PixelFormat::I444.buffer_size(width, height)];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::I444,
        PixelFormatPlanes::infer_i444(&mut i444[..], width, height),
        width,
        height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::I444,
        PixelFormatPlanes::infer_i444(&i444[..], width, height),
        width,
        height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::BT709,
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/I444_TO_RGBA.png").unwrap();
}

#[test]
fn rgba8_to_nv12_and_back_ictcp_pq() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = vec![0u8; PixelFormat::NV12.buffer_size(width, height) * 2];

    let src = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&mut nv12[..], width, height),
        width,
        height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::ICtCpPQ,
            transfer: ColorTransfer::BT2100PQ,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        16,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let src = Image::new(
        PixelFormat::NV12,
        PixelFormatPlanes::infer_nv12(&nv12[..], width, height),
        width,
        height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::ICtCpPQ,
            transfer: ColorTransfer::BT2100PQ,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
        16,
    )
    .unwrap();

    let dst = Image::new(
        PixelFormat::RGBA,
        PixelFormatPlanes::RGBA(&mut rgba8[..]),
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/NV12_TO_RGBA_ICTCP_PQ.png").unwrap();
}

#[test]
fn rgba8_to_nv12_and_back_ictcp_hlg() {
    let (mut rgba8, width, height) = make_rgba8_image(ColorPrimaries::BT709, ColorTransfer::Linear);

    let mut nv12 = vec![0u8; PixelFormat::NV12.buffer_size(width, height)];

    let mut rgb = Image::new(
        PixelFormat::RGBA,
        vec![&mut rgba8[..]],
        vec![width * 4],
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
    )
    .unwrap();

    let mut nv12 = Image::new(
        PixelFormat::NV12,
        infer_nv12(&mut nv12[..], width, height).into(),
        vec![width, width],
        width,
        height,
        ColorInfo::YUV(YuvColorInfo {
            space: ColorSpace::ICtCpHLG,
            transfer: ColorTransfer::BT2100HLG,
            primaries: ColorPrimaries::BT709,
            full_range: false,
        }),
    )
    .unwrap();

    println!("1");
    convert(&rgb, &mut nv12).unwrap();

    println!("2");
    convert(&nv12, &mut rgb).unwrap();

    println!("3");
    let buffer =
        image::ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as _, height as _, rgba8).unwrap();

    buffer.save("tests/NV12_TO_RGBA_ICTCP_HLG.png").unwrap();
}
*/
#[test]
fn yuyv_to_rgb() {
    let mut yuyv = std::fs::read("tests/data/switch.yuyv").unwrap();

    // YUYV -> RGB
    let mut yuyv = Image::new(
        PixelFormat::YUYV,
        vec![&mut yuyv[..]],
        vec![1920 * 2],
        1920,
        1080,
        ColorInfo::YUV(YuvColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            space: ColorSpace::BT601,
            full_range: false,
        }),
    )
    .unwrap();

    // YUYV

    let mut rgb = Image::blank(
        PixelFormat::RGB,
        1920,
        1080,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
        }),
    );

    convert(&yuyv, &mut rgb).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        1920,
        1080,
        rgb.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/YUYV_TO_RGB.png").unwrap();

    // // RGB -> YUYV

    convert(&rgb, &mut yuyv).unwrap();

    // // YUYV -> RGB

    convert(&yuyv, &mut rgb).unwrap();

    let buffer = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
        1920,
        1080,
        rgb.planes().next().unwrap().0.to_vec(),
    )
    .unwrap();

    buffer.save("tests/RGB_TO_YUYV.png").unwrap();
}
/*
#[test]
fn windows_offsets() {
    let color = YuvColorInfo {
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        space: ColorSpace::BT709,
        full_range: false,
    };

    let (i420_src, width, height) = make_i420_image(color);

    let mut dst_image_buffer = vec![128u8; PixelFormat::I420.buffer_size(1920, 1080)];

    for x in 2..3 {
        for y in 2..3 {
            convert(
                Image::new(
                    PixelFormat::I420,
                    PixelFormatPlanes::infer_i420(i420_src.as_slice(), width, height),
                    dbg!(width),
                    dbg!(height),
                    ColorInfo::YUV(color),
                    8,
                )
                .unwrap(),
                Image::new(
                    PixelFormat::I420,
                    PixelFormatPlanes::infer_i420(dst_image_buffer.as_mut_slice(), 1920, 1080),
                    1920,
                    1080,
                    ColorInfo::YUV(color),
                    8,
                )
                .unwrap()
                .with_window(Window {
                    x: dbg!(640 * x),
                    y: dbg!(360 * y),
                    width,
                    height,
                })
                .unwrap(),
            )
            .unwrap();
        }
    }
    // I420 -> RGB
    let src = Image::new(
        PixelFormat::I420,
        PixelFormatPlanes::infer_i420(dst_image_buffer.as_slice(), 1920, 1080),
        1920,
        1080,
        ColorInfo::YUV(color),
        8,
    )
    .unwrap();

    let mut rgb_dst = vec![0u8; PixelFormat::RGB.buffer_size(1920, 1080)];
    let dst = Image::new(
        PixelFormat::RGB,
        PixelFormatPlanes::RGB(&mut rgb_dst[..]),
        1920,
        1080,
        ColorInfo::RGB(RgbColorInfo {
            transfer: ColorTransfer::SRGB,
            primaries: ColorPrimaries::BT709,
        }),
        8,
    )
    .unwrap();

    convert(src, dst).unwrap();

    let buffer =
        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(1920, 1080, rgb_dst.clone()).unwrap();

    buffer.save("tests/WINDOW_OFFSETS.png").unwrap();
}
 */
