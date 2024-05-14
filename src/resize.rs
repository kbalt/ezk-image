use crate::{Image, PixelFormatPlanes, Primitive, Window};
use std::error::Error;
use std::fmt;

#[cfg(feature = "multi-thread")]
use rayon::scope;
#[cfg(not(feature = "multi-thread"))]
use rayon_stub::scope;

pub use fir::{Filter, FilterType, ResizeAlg};

/// Everything that can go wrong when calling [`Resizer::resize`]
#[derive(Debug, PartialEq)]
pub enum ResizeError {
    DifferentFormats,
}

impl fmt::Display for ResizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResizeError::DifferentFormats => write!(
                f,
                "source and destination images have different pixel formats"
            ),
        }
    }
}

impl Error for ResizeError {}

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
        self.fir.resize_with(N, fir::Resizer::new);

        let mut iter = self.fir.iter_mut();

        std::array::from_fn(|_| iter.next().expect("just resized to the correct len"))
    }

    /// Resize an image. `src` and `dst` must have the same pixel format.
    ///
    /// Color characteristics of the images are ignored.
    pub fn resize<P>(&mut self, src: Image<&[P]>, dst: Image<&mut [P]>) -> Result<(), ResizeError>
    where
        P: Primitive,
    {
        if src.format != dst.format {
            return Err(ResizeError::DifferentFormats);
        }

        let alg = self.alg;

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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                            alg,
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
                    alg,
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
                    alg,
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
        alg: ResizeAlg,

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
        Px: fir::PixelTrait,
    {
        // Safety:
        // P is either u8 or u16, so transmuting to u8 isn't an issue
        let src_slice = unsafe { src.align_to::<u8>().1 };
        let dst_slice = unsafe { dst.align_to_mut::<u8>().1 };

        let src_view =
            fir::images::ImageRef::new(src_width, src_height, src_slice, Px::pixel_type())
                .expect("ImageBuffer invariants should have been checked beforehand");

        let src_image = if let Some(window) = src_window {
            fir::images::CroppedImage::new(
                &src_view,
                window.x as u32,
                window.y as u32,
                window.width as u32,
                window.height as u32,
            )
        } else {
            fir::images::CroppedImage::new(&src_view, 0, 0, src_width, src_height)
        };

        let mut dst_view =
            fir::images::Image::from_slice_u8(dst_width, dst_height, dst_slice, Px::pixel_type())
                .expect("ImageBuffer invariants should have been checked beforehand");

        let dst_image = if let Some(window) = dst_window {
            fir::images::CroppedImageMut::new(
                &mut dst_view,
                window.x as u32,
                window.y as u32,
                window.width as u32,
                window.height as u32,
            )
        } else {
            fir::images::CroppedImageMut::new(&mut dst_view, 0, 0, dst_width, dst_height)
        };

        fir_resizer
            .resize(
                &src_image.expect("Crop must be checked by Image"),
                &mut dst_image.expect("Crop must be checked by Image"),
                Some(&fir::ResizeOptions {
                    algorithm: alg,
                    cropping: fir::SrcCropping::None,
                    mul_div_alpha: false,
                }),
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
