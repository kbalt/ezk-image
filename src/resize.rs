use crate::{primitive::PrimitiveInternal, Image, PixelFormatPlanes};

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
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_v,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
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
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32,
                dst.height as u32 / 2,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32 / 2,
                dst_v,
                dst.width as u32,
                dst.height as u32 / 2,
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
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32,
                dst_u,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<P, P::FirPixel1>(
                &mut resizer,
                src_v,
                src.width as u32,
                src.height as u32,
                dst_v,
                dst.width as u32,
                dst.height as u32,
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
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<P, P::FirPixel2>(
                &mut resizer,
                src_uv,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_uv,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );
        }
        (PixelFormatPlanes::RGB(src_rgb), PixelFormatPlanes::RGB(dst_rgb)) => {
            resize_plane::<P, P::FirPixel3>(
                &mut resizer,
                src_rgb,
                src.width as u32,
                src.height as u32,
                dst_rgb,
                dst.width as u32,
                dst.height as u32,
            );
        }
        (PixelFormatPlanes::RGBA(src_rgba), PixelFormatPlanes::RGBA(dst_rgba)) => {
            resize_plane::<P, P::FirPixel4>(
                &mut resizer,
                src_rgba,
                src.width as u32,
                src.height as u32,
                dst_rgba,
                dst.width as u32,
                dst.height as u32,
            );
        }
        _ => unreachable!(),
    }
}

fn resize_plane<P, Px>(
    resizer: &mut fir::Resizer,

    src: &[P],
    src_width: u32,
    src_height: u32,

    dst: &mut [P],
    dst_width: u32,
    dst_height: u32,
) where
    P: PrimitiveInternal,
    Px: fir::pixels::PixelExt,
    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, Px>>,
    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, Px>>,
{
    let src_slice = unsafe { src.align_to::<u8>().1 };
    let dst_slice = unsafe { dst.align_to_mut::<u8>().1 };

    let src_view = fir::ImageView::<Px>::from_buffer(
        src_width.try_into().unwrap(),
        src_height.try_into().unwrap(),
        src_slice,
    )
    .unwrap();

    let dst_view = fir::ImageViewMut::<Px>::from_buffer(
        dst_width.try_into().unwrap(),
        dst_height.try_into().unwrap(),
        dst_slice,
    )
    .unwrap();

    resizer
        .resize(
            &fir::DynamicImageView::from(src_view),
            &mut fir::DynamicImageViewMut::from(dst_view),
        )
        .unwrap();
}
