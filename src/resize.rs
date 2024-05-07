use crate::{primitive::PrimitiveInternal, Image, PixelFormatPlanes, Rect};
use std::num::NonZeroU32;

#[allow(private_bounds)]
pub fn scale<P>(src: Image<&[P]>, dst: Image<&mut [P]>)
where
    P: PrimitiveInternal,

    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, P::FirPixel1>>
        + From<fir::ImageView<'a, P::FirPixel2>>
        + From<fir::ImageView<'a, P::FirPixel3>>
        + From<fir::ImageView<'a, P::FirPixel4>>,

    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, P::FirPixel1>>
        + From<fir::ImageViewMut<'a, P::FirPixel2>>
        + From<fir::ImageViewMut<'a, P::FirPixel3>>
        + From<fir::ImageViewMut<'a, P::FirPixel4>>,
{
    assert_eq!(src.format, dst.format);

    let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Bilinear));

    match (src.planes, dst.planes) {
        (
            PixelFormatPlanes::I420 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I420 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_y,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32 / 2,
                src.height as u32 / 2,
                src.window.map(window_div_2x2),
                dst_u,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
                dst.window.map(window_div_2x2),
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32 / 2,
                src.height as u32 / 2,
                src.window.map(window_div_2x2),
                dst_v,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
                dst.window.map(window_div_2x2),
            );
        }
        (
            PixelFormatPlanes::I422 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I422 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_y,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32 / 2,
                src.window.map(window_div_1x2),
                dst_u,
                dst.width as u32,
                dst.height as u32 / 2,
                dst.window.map(window_div_1x2),
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32 / 2,
                src.window.map(window_div_1x2),
                dst_v,
                dst.width as u32,
                dst.height as u32 / 2,
                dst.window.map(window_div_1x2),
            );
        }
        (
            PixelFormatPlanes::I444 {
                y: src_y,
                u: src_u,
                v: src_v,
            },
            PixelFormatPlanes::I444 {
                y: dst_y,
                u: dst_u,
                v: dst_v,
            },
        ) => {
            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_y,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_u,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_v,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );
        }
        (
            PixelFormatPlanes::NV12 {
                y: src_y,
                uv: src_uv,
            },
            PixelFormatPlanes::NV12 {
                y: dst_y,
                uv: dst_uv,
            },
        ) => {
            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_y,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );

            resize_plane::<P, P::FirPixel2>(
                &mut resizer,
                src_uv,
                src.width as u32 / 2,
                src.height as u32 / 2,
                src.window.map(window_div_2x2),
                dst_uv,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
                dst.window.map(window_div_2x2),
            );
        }
        (PixelFormatPlanes::RGB(src_rgb), PixelFormatPlanes::RGB(dst_rgb)) => {
            resize_plane::<P, P::FirPixel3>(
                &mut resizer,
                src_rgb,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_rgb,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );
        }
        (PixelFormatPlanes::RGBA(src_rgba), PixelFormatPlanes::RGBA(dst_rgba)) => {
            resize_plane::<P, P::FirPixel4>(
                &mut resizer,
                src_rgba,
                src.width as u32,
                src.height as u32,
                src.window,
                dst_rgba,
                dst.width as u32,
                dst.height as u32,
                dst.window,
            );
        }
        _ => unreachable!(),
    }
}

fn window_div_1x2(w: Rect) -> Rect {
    Rect {
        x: w.x,
        y: w.y / 2,
        width: w.width,
        height: w.height / 2,
    }
}

fn window_div_2x2(w: Rect) -> Rect {
    Rect {
        x: w.x / 2,
        y: w.y / 2,
        width: w.width / 2,
        height: w.height / 2,
    }
}

#[allow(clippy::too_many_arguments)]
fn resize_plane<P, Px>(
    resizer: &mut fir::Resizer,

    src: &[P],
    src_width: u32,
    src_height: u32,
    src_window: Option<Rect>,

    dst: &mut [P],
    dst_width: u32,
    dst_height: u32,
    dst_window: Option<Rect>,
) where
    P: PrimitiveInternal,
    Px: fir::pixels::PixelExt,
    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, Px>>,
    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, Px>>,
{
    // Safety:
    // P is either u8 or u16, so transmuting to u8 isn't an issue
    let src_slice = unsafe { src.align_to::<u8>().1 };
    let dst_slice = unsafe { dst.align_to_mut::<u8>().1 };

    let mut src_view = fir::ImageView::<Px>::from_buffer(
        src_width.try_into().unwrap(),
        src_height.try_into().unwrap(),
        src_slice,
    )
    .unwrap();

    if let Some(src_window) = src_window {
        src_view
            .set_crop_box(fir::CropBox {
                left: src_window.x as f64,
                top: src_window.y as f64,
                width: src_window.width as f64,
                height: src_window.height as f64,
            })
            .unwrap();
    }

    let mut dst_view = fir::ImageViewMut::<Px>::from_buffer(
        dst_width.try_into().unwrap(),
        dst_height.try_into().unwrap(),
        dst_slice,
    )
    .unwrap();

    if let Some(dst_window) = dst_window {
        dst_view = dst_view
            .crop(
                dst_window.x as u32,
                dst_window.y as u32,
                NonZeroU32::try_from(dst_window.width as u32).unwrap(),
                NonZeroU32::try_from(dst_window.height as u32).unwrap(),
            )
            .unwrap();
    }

    resizer
        .resize(
            &fir::DynamicImageView::from(src_view),
            &mut fir::DynamicImageViewMut::from(dst_view),
        )
        .unwrap();
}
