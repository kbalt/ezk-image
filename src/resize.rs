use crate::{ImageMut, ImageRef, PixelFormat};
use fir::{IntoImageView, IntoImageViewMut, pixels::InnerPixel};
#[cfg(feature = "multi-thread")]
use rayon::scope;
#[cfg(not(feature = "multi-thread"))]
use rayon_stub::scope;
use std::{cmp, error::Error, fmt, marker::PhantomData};

pub use fir::{Filter, FilterType, ResizeAlg};

/// Everything that can go wrong when calling [`Resizer::resize`]
#[derive(Debug, PartialEq)]
pub enum ResizeError {
    DifferentFormats(PixelFormat, PixelFormat),
}

impl fmt::Display for ResizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResizeError::DifferentFormats(src, dst) => write!(
                f,
                "source and destination images have different pixel formats (source image is {src:?} and destination is {dst:?})"
            ),
        }
    }
}

impl Error for ResizeError {}

/// Wrapper over [`fast_image_resize`](fir)
#[derive(Clone)]
pub struct Resizer {
    alg: fir::ResizeAlg,
    fir: Vec<fir::Resizer>,
}

impl Resizer {
    pub fn new(alg: fir::ResizeAlg) -> Self {
        Self { alg, fir: vec![] }
    }

    /// Resize an image. `src` and `dst` must have the same pixel format.
    ///
    /// Color characteristics of the images are ignored.
    pub fn resize(
        &mut self,
        src: &dyn ImageRef,
        dst: &mut dyn ImageMut,
    ) -> Result<(), ResizeError> {
        if src.format() != dst.format() {
            return Err(ResizeError::DifferentFormats(src.format(), dst.format()));
        }

        let alg = self.alg;

        let desc = src.format().plane_desc();

        let src_width = src.width();
        let dst_width = dst.width();

        let src_planes: Vec<(&[u8], usize)> = src.planes().collect();
        let dst_planes: Vec<(&mut [u8], usize)> = dst.planes_mut().collect();

        self.fir.resize_with(
            cmp::max(self.fir.len(), src_planes.len()),
            fir::Resizer::new,
        );

        scope(|s| {
            for ((plane_desc, ((src_plane, src_stride), (dst_plane, dst_stride))), fir_resizer) in
                desc.iter()
                    .zip(src_planes.into_iter().zip(dst_planes))
                    .zip(&mut self.fir)
            {
                s.spawn(move |_| {
                    let src_fir_width = plane_desc.width_op.op(src_width)
                        / (plane_desc.pixel_type.size() / plane_desc.bytes_per_primitive);
                    let dst_fir_width = plane_desc.width_op.op(dst_width)
                        / (plane_desc.pixel_type.size() / plane_desc.bytes_per_primitive);

                    let src_height = src_plane.len() / src_stride;
                    let dst_height = dst_plane.len() / dst_stride;

                    let src = FirAdapterIntoImageView {
                        pixel_type: plane_desc.pixel_type,
                        width: src_fir_width,
                        height: src_height,
                        stride: src_stride,
                        plane: src_plane,
                    };

                    let mut dst = FirAdapterIntoImageView {
                        pixel_type: plane_desc.pixel_type,
                        width: dst_fir_width,
                        height: dst_height,
                        stride: dst_stride,
                        plane: dst_plane,
                    };

                    fir_resizer
                        .resize(
                            &src,
                            &mut dst,
                            Some(&fir::ResizeOptions {
                                algorithm: alg,
                                cropping: fir::SrcCropping::None,
                                mul_div_alpha: false,
                            }),
                        )
                        .expect(
                            "Pixel type must be assured to be the same before calling fir's resize",
                        );
                })
            }
        });

        Ok(())
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

struct FirAdapterIntoImageView<T> {
    pixel_type: fir::PixelType,
    width: usize,
    height: usize,
    stride: usize,
    plane: T,
}

impl<T: AsRef<[u8]>> IntoImageView for FirAdapterIntoImageView<T> {
    fn pixel_type(&self) -> Option<fir::PixelType> {
        Some(self.pixel_type)
    }

    fn width(&self) -> u32 {
        self.width as u32
    }

    fn height(&self) -> u32 {
        self.height as u32
    }

    fn image_view<P: fir::PixelTrait>(&self) -> Option<impl fir::ImageView<Pixel = P>> {
        if P::pixel_type() != self.pixel_type {
            return None;
        }

        Some(FirAdapterImageView {
            width: self.width,
            height: self.height,
            stride: self.stride,
            plane: self.plane.as_ref(),
            _pixel_type: PhantomData,
        })
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]> + Send + Sync> IntoImageViewMut for FirAdapterIntoImageView<T> {
    fn image_view_mut<P: fir::PixelTrait>(&mut self) -> Option<impl fir::ImageViewMut<Pixel = P>> {
        if P::pixel_type() != self.pixel_type {
            return None;
        }

        Some(FirAdapterImageView {
            width: self.width,
            height: self.height,
            stride: self.stride,
            plane: self.plane.as_mut(),
            _pixel_type: PhantomData,
        })
    }
}

struct FirAdapterImageView<T, P> {
    width: usize,
    height: usize,
    stride: usize,
    plane: T,
    _pixel_type: PhantomData<P>,
}

unsafe impl<T: AsRef<[u8]> + Send + Sync, P: InnerPixel> fir::ImageView
    for FirAdapterImageView<T, P>
{
    type Pixel = P;

    fn width(&self) -> u32 {
        self.width as u32
    }

    fn height(&self) -> u32 {
        self.height as u32
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        self.plane
            .as_ref()
            .chunks_exact(self.stride)
            .skip(start_row as usize)
            .map(|row| {
                let row = &row[..self.width * P::pixel_type().size()];

                let (unwanted1, row, unwanted2) = unsafe { row.align_to::<P>() };

                assert_eq!(row.len(), self.width);
                assert!(unwanted1.is_empty());
                assert!(unwanted2.is_empty());

                row
            })
    }
}

unsafe impl<T: AsRef<[u8]> + AsMut<[u8]> + Send + Sync, P: InnerPixel> fir::ImageViewMut
    for FirAdapterImageView<T, P>
{
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        self.plane
            .as_mut()
            .chunks_exact_mut(self.stride)
            .skip(start_row as usize)
            .map(|row| {
                let row = &mut row[..self.width * P::pixel_type().size()];

                let (unwanted1, row, unwanted2) = unsafe { row.align_to_mut::<P>() };

                assert_eq!(row.len(), self.width);
                assert!(unwanted1.is_empty());
                assert!(unwanted2.is_empty());

                row
            })
    }
}
