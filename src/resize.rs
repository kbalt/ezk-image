use crate::{bits::BitsInternal, endian::Endian, Destination, PixelFormatPlanes, Source};

#[allow(private_bounds)]
pub fn scale<B>(src: Source<'_, B>, dst: Destination<'_, B>)
where
    B: BitsInternal,

    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, B::FirPixel1>>
        + From<fir::ImageView<'a, B::FirPixel2>>
        + From<fir::ImageView<'a, B::FirPixel3>>
        + From<fir::ImageView<'a, B::FirPixel4>>,

    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, B::FirPixel1>>
        + From<fir::ImageViewMut<'a, B::FirPixel2>>
        + From<fir::ImageViewMut<'a, B::FirPixel3>>
        + From<fir::ImageViewMut<'a, B::FirPixel4>>,
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
            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32 / 2,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32 / 2,
                dst.height as u32 / 2,
            );

            resize_plane::<B, B::FirPixel1>(
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
            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32 / 2,
                dst_u,
                dst.width as u32,
                dst.height as u32 / 2,
            );

            resize_plane::<B, B::FirPixel1>(
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
            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_u,
                src.width as u32,
                src.height as u32,
                dst_u,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<B, B::FirPixel1>(
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
            resize_plane::<B, B::FirPixel1>(
                &mut resizer,
                src_y,
                src.width as u32,
                src.height as u32,
                dst_y,
                dst.width as u32,
                dst.height as u32,
            );

            resize_plane::<B, B::FirPixel2>(
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
            resize_plane::<B, B::FirPixel3>(
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
            resize_plane::<B, B::FirPixel4>(
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

fn resize_plane<B, Px>(
    resizer: &mut fir::Resizer,

    src: &[B::Primitive],
    src_width: u32,
    src_height: u32,

    dst: &mut [B::Primitive],
    dst_width: u32,
    dst_height: u32,
) where
    B: BitsInternal,
    Px: fir::pixels::PixelExt,
    for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, Px>>,
    for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, Px>>,
{
    let mut src_copy;

    let src_slice = if B::Endian::IS_NATIVE {
        unsafe { src.align_to::<u8>().1 }
    } else {
        // Wrong endianess for FIR, convert first then resize
        src_copy = src.to_vec();
        B::swap_bytes(&mut src_copy);
        unsafe { src_copy.align_to::<u8>().1 }
    };

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

    if !B::Endian::IS_NATIVE {
        // Swap bytes back to original endianess
        B::swap_bytes(dst);
    }
}
