use crate::{Image, PixelFormatPlanes, Primitive, Window};
use std::num::NonZeroU32;

#[cfg(feature = "multi-thread")]
use rayon::scope;
#[cfg(not(feature = "multi-thread"))]
use rayon_stub::scope;

/// Everything that can go wrong when calling [`Resizer::resize`]
#[derive(Debug, thiserror::Error)]
pub enum ResizeError {
    #[error("source and destination images have different pixel formats")]
    DifferentFormats,
}

/// Wrapper over [`fast_image_resize`](fir) to resize [`Image`]s
#[derive(Clone)]
pub struct Resizer {
    alg: fir::ResizeAlg,
    fir: Vec<fir::Resizer>,
}

impl Resizer {
    pub fn new(alg: fir::ResizeAlg) -> Self {
        Self { alg, fir: vec![] }
    }

    fn ensure_resizer_len<const N: usize>(&mut self) -> [&mut fir::Resizer; N] {
        self.fir.resize_with(N, || fir::Resizer::new(self.alg));

        let mut iter = self.fir.iter_mut();

        std::array::from_fn(|_| iter.next().expect("just resized to the correct len"))
    }

    /// Resize an image. `src` and `dst` must have the same pixel format.
    ///
    /// Transfer characteristics of the source image are ignored.
    #[allow(private_bounds)]
    pub fn resize<P>(&mut self, src: Image<&[P]>, dst: Image<&mut [P]>) -> Result<(), ResizeError>
    where
        P: Primitive,
        for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, P::FirPixel1>>
            + From<fir::ImageView<'a, P::FirPixel2>>
            + From<fir::ImageView<'a, P::FirPixel3>>
            + From<fir::ImageView<'a, P::FirPixel4>>,
        for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, P::FirPixel1>>
            + From<fir::ImageViewMut<'a, P::FirPixel2>>
            + From<fir::ImageViewMut<'a, P::FirPixel3>>
            + From<fir::ImageViewMut<'a, P::FirPixel4>>,
    {
        if src.format != dst.format {
            return Err(ResizeError::DifferentFormats);
        }

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
                let [fir_resizer0, fir_resizer1, fir_resizer2] = self.ensure_resizer_len::<3>();

                scope(|s| {
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer0,
                            src_y,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_y,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });

                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer1,
                            src_u,
                            src.width as u32 / 2,
                            src.height as u32 / 2,
                            src.window.map(window_div_2x2),
                            dst_u,
                            dst.width as u32 / 2,
                            dst.height as u32 / 2,
                            dst.window.map(window_div_2x2),
                        );
                    });

                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer2,
                            src_v,
                            src.width as u32 / 2,
                            src.height as u32 / 2,
                            src.window.map(window_div_2x2),
                            dst_v,
                            dst.width as u32 / 2,
                            dst.height as u32 / 2,
                            dst.window.map(window_div_2x2),
                        );
                    });
                });
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
                let [fir_resizer0, fir_resizer1, fir_resizer2] = self.ensure_resizer_len::<3>();

                scope(|s| {
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer0,
                            src_y,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_y,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer1,
                            src_u,
                            src.width as u32,
                            src.height as u32 / 2,
                            src.window.map(window_div_1x2),
                            dst_u,
                            dst.width as u32,
                            dst.height as u32 / 2,
                            dst.window.map(window_div_1x2),
                        );
                    });
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer2,
                            src_v,
                            src.width as u32,
                            src.height as u32 / 2,
                            src.window.map(window_div_1x2),
                            dst_v,
                            dst.width as u32,
                            dst.height as u32 / 2,
                            dst.window.map(window_div_1x2),
                        );
                    });
                });
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
                let [fir_resizer0, fir_resizer1, fir_resizer2] = self.ensure_resizer_len::<3>();

                scope(|s| {
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer0,
                            src_y,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_y,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });

                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer1,
                            src_u,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_u,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });

                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer2,
                            src_v,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_v,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });
                })
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
                let [fir_resizer0, fir_resizer1] = self.ensure_resizer_len::<2>();

                scope(|s| {
                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel1>(
                            fir_resizer0,
                            src_y,
                            src.width as u32,
                            src.height as u32,
                            src.window,
                            dst_y,
                            dst.width as u32,
                            dst.height as u32,
                            dst.window,
                        );
                    });

                    s.spawn(move |_| {
                        Self::resize_plane::<P, P::FirPixel2>(
                            fir_resizer1,
                            src_uv,
                            src.width as u32 / 2,
                            src.height as u32 / 2,
                            src.window.map(window_div_2x2),
                            dst_uv,
                            dst.width as u32 / 2,
                            dst.height as u32 / 2,
                            dst.window.map(window_div_2x2),
                        );
                    });
                });
            }
            (PixelFormatPlanes::RGB(src_rgb), PixelFormatPlanes::RGB(dst_rgb)) => {
                let [fir_resizer] = self.ensure_resizer_len::<1>();

                Self::resize_plane::<P, P::FirPixel3>(
                    fir_resizer,
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
                let [fir_resizer] = self.ensure_resizer_len::<1>();

                Self::resize_plane::<P, P::FirPixel4>(
                    fir_resizer,
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
            _ => return Err(ResizeError::DifferentFormats),
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn resize_plane<P, Px>(
        fir_resizer: &mut fir::Resizer,

        src: &[P],
        src_width: u32,
        src_height: u32,
        src_window: Option<Window>,

        dst: &mut [P],
        dst_width: u32,
        dst_height: u32,
        dst_window: Option<Window>,
    ) where
        P: Primitive,
        Px: fir::pixels::PixelExt,
        for<'a> fir::DynamicImageView<'a>: From<fir::ImageView<'a, Px>>,
        for<'a> fir::DynamicImageViewMut<'a>: From<fir::ImageViewMut<'a, Px>>,
    {
        // Safety:
        // P is either u8 or u16, so transmuting to u8 isn't an issue
        let src_slice = unsafe { src.align_to::<u8>().1 };
        let dst_slice = unsafe { dst.align_to_mut::<u8>().1 };

        let mut src_view = fir::ImageView::<Px>::from_buffer(
            src_width
                .try_into()
                .expect("Image::new() must prevent non-zero width"),
            src_height
                .try_into()
                .expect("Image::new() must prevent non-zero height"),
            src_slice,
        )
        .expect("ImageBuffer invariants should have been checked beforehand");

        if let Some(src_window) = src_window {
            src_view
                .set_crop_box(fir::CropBox {
                    left: src_window.x as f64,
                    top: src_window.y as f64,
                    width: src_window.width as f64,
                    height: src_window.height as f64,
                })
                .expect("window size is already checked when creating Image");
        }

        let mut dst_view = fir::ImageViewMut::<Px>::from_buffer(
            dst_width.try_into().unwrap(),
            dst_height.try_into().unwrap(),
            dst_slice,
        )
        .expect("ImageBuffer invariants should have been checked beforehand");

        if let Some(dst_window) = dst_window {
            dst_view = dst_view
                .crop(
                    dst_window.x as u32,
                    dst_window.y as u32,
                    NonZeroU32::try_from(dst_window.width as u32)
                        .expect("Image::new() must prevent non-zero width"),
                    NonZeroU32::try_from(dst_window.height as u32)
                        .expect("Image::new() must prevent non-zero height"),
                )
                .expect("window size is already checked when creating Image");
        }

        fir_resizer
            .resize(
                &fir::DynamicImageView::from(src_view),
                &mut fir::DynamicImageViewMut::from(dst_view),
            )
            .expect("Pixel type must be assured to be the same before calling fir's resize");
    }
}

fn window_div_1x2(w: Window) -> Window {
    Window {
        x: w.x,
        y: w.y / 2,
        width: w.width,
        height: w.height / 2,
    }
}

fn window_div_2x2(w: Window) -> Window {
    Window {
        x: w.x / 2,
        y: w.y / 2,
        width: w.width / 2,
        height: w.height / 2,
    }
}

#[cfg(not(feature = "multi-thread"))]
mod rayon_stub {
    pub(super) struct Scope {}

    impl Scope {
        pub(super) fn spawn(&self, f: impl FnOnce(&Scope)) {
            scope(f)
        }
    }

    pub(super) fn scope(f: impl FnOnce(&Scope)) {
        f(&Scope {})
    }
}
