use ezk_image::{
    convert_multi_thread, resize::Resizer, ColorInfo, ColorPrimaries, ColorSpace, ColorTransfer,
    Image, ImageRef, PixelFormat, RgbColorInfo, YuvColorInfo,
};
use fir::ResizeAlg;
use image::Rgb;

fn make_rgba8_image(
    width: usize,
    height: usize,
    primaries: ColorPrimaries,
    transfer: ColorTransfer,
) -> Image<Vec<u8>> {
    let mat = primaries.xyz_to_rgb_mat();

    let mut out = Vec::with_capacity(width * height * 4);

    let y = 0.4;

    for x in 0..height {
        let x = (x as f32) / width as f32;

        for z in 0..width {
            let z = (z as f32) / height as f32;

            let r = x * mat[0][0] + y * mat[1][0] + z * mat[2][0];
            let g = x * mat[0][1] + y * mat[1][1] + z * mat[2][1];
            let b = x * mat[0][2] + y * mat[1][2] + z * mat[2][2];

            out.push((transfer.linear_to_scaled(r) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(g) * u8::MAX as f32) as u8);
            out.push((transfer.linear_to_scaled(b) * u8::MAX as f32) as u8);
            out.push(u8::MAX);
        }
    }

    Image::from_buffer(
        PixelFormat::RGBA,
        out,
        None,
        width,
        height,
        ColorInfo::RGB(RgbColorInfo {
            transfer,
            primaries,
        }),
    )
    .unwrap()
}

#[test]
fn all_from_rgb_and_back() {
    let color = ColorInfo::YUV(YuvColorInfo {
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        space: ColorSpace::BT709,
        full_range: false,
    });

    let rgba = make_rgba8_image(500, 500, ColorPrimaries::BT709, ColorTransfer::Linear);

    for format in PixelFormat::variants() {
        let mut tmp_small = Image::blank(format, rgba.width(), rgba.height(), color);
        convert_multi_thread(&rgba, &mut tmp_small).unwrap();

        let mut tmp_big = Image::blank(format, 1920, 1080, color);

        Resizer::new(ResizeAlg::Nearest)
            .resize(&tmp_small, &mut tmp_big)
            .unwrap();

        let mut rgb = Image::blank(PixelFormat::RGB, tmp_big.width(), tmp_big.height(), color);

        convert_multi_thread(&tmp_big, &mut rgb).unwrap();

        image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(
            rgb.width() as _,
            rgb.height() as _,
            rgb.planes().next().unwrap().0.to_vec(),
        )
        .unwrap()
        .save(format!("tests/ALL_{format:?}.png"))
        .unwrap();
    }
}
