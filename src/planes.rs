use crate::{PixelFormat, StrictApi, Window};
use std::mem::take;

/// All supported image plane formats
///
/// Has a bunch of convenience functions to split buffers up into image planes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormatPlanes<S: AnySlice> {
    /// See [`PixelFormat::I420`]
    I420 { y: S, u: S, v: S },

    /// See [`PixelFormat::I422`]
    I422 { y: S, u: S, v: S },

    /// See [`PixelFormat::I444`]
    I444 { y: S, u: S, v: S },

    /// See [`PixelFormat::NV12`]
    NV12 { y: S, uv: S },

    /// See [`PixelFormat::YUYV`]
    YUYV(S),

    /// See [`PixelFormat::RGB`] and [`PixelFormat::BGR`],
    RGB(S),

    /// See [`PixelFormat::RGBA`] and [`PixelFormat::BGRA`]
    RGBA(S),
}

impl<S: AnySlice> PixelFormatPlanes<S> {
    /// Returns if the given planes are large enough for the given dimensions
    #[deny(clippy::arithmetic_side_effects)]
    pub fn bounds_check(&self, width: usize, height: usize) -> bool {
        let Some(n_pixels) = width.checked_mul(height) else {
            return false;
        };

        match self {
            Self::I420 { y, u, v } => {
                let uv_req_len = n_pixels / 4;

                n_pixels <= y.slice_len()
                    && uv_req_len <= u.slice_len()
                    && uv_req_len <= v.slice_len()
            }
            Self::I422 { y, u, v } => {
                let uv_req_len = n_pixels / 2;

                n_pixels <= y.slice_len()
                    && uv_req_len <= u.slice_len()
                    && uv_req_len <= v.slice_len()
            }
            Self::I444 { y, u, v } => {
                n_pixels <= y.slice_len() && n_pixels <= u.slice_len() && n_pixels <= v.slice_len()
            }
            Self::NV12 { y, uv } => {
                let uv_req_len = n_pixels / 2;

                n_pixels <= y.slice_len() && uv_req_len <= uv.slice_len()
            }
            Self::YUYV(buf) => {
                let Some(n_bytes) = n_pixels.checked_mul(2) else {
                    return false;
                };

                n_bytes <= buf.slice_len()
            }
            Self::RGB(buf) => {
                let Some(n_bytes) = n_pixels.checked_mul(3) else {
                    return false;
                };

                n_bytes <= buf.slice_len()
            }
            Self::RGBA(buf) => {
                let Some(n_bytes) = n_pixels.checked_mul(4) else {
                    return false;
                };

                n_bytes <= buf.slice_len()
            }
        }
    }

    /// Infer the planes for an image in the given format using the given dimensions
    ///
    /// # Panics
    ///
    /// If `buf` is too small for the given dimensions this function will panic
    #[deny(clippy::arithmetic_side_effects)]
    pub fn infer(format: PixelFormat, buf: S, width: usize, height: usize) -> Self {
        match format {
            PixelFormat::I420 => Self::infer_i420(buf, width, height),
            PixelFormat::I422 => Self::infer_i422(buf, width, height),
            PixelFormat::I444 => Self::infer_i444(buf, width, height),
            PixelFormat::NV12 => Self::infer_nv12(buf, width, height),
            PixelFormat::YUYV => Self::YUYV(buf),
            PixelFormat::RGBA | PixelFormat::BGRA => Self::RGBA(buf),
            PixelFormat::RGB | PixelFormat::BGR => Self::RGB(buf),
        }
    }

    /// Infer the planes for a full I420 image using the given dimensions
    ///
    /// # Panics
    ///
    /// If `buf` is too small for the given dimensions this function will panic
    #[deny(clippy::arithmetic_side_effects)]
    pub fn infer_i420(buf: S, width: usize, height: usize) -> Self {
        let (y, tmp) = buf.slice_split_at(width.strict_mul_(height));
        let (u, v) = tmp.slice_split_at(width.strict_mul_(height) / 4);

        Self::I420 { y, u, v }
    }

    /// Infer the planes for a full NV12 image using the given dimensions
    ///
    /// # Panics
    ///
    /// If `buf` is too small for the given dimensions this function will panic
    #[deny(clippy::arithmetic_side_effects)]
    pub fn infer_nv12(buf: S, width: usize, height: usize) -> Self {
        let (y, uv) = buf.slice_split_at(width.strict_mul_(height));

        assert!(uv.slice_len() >= width.strict_mul_(height) / 2);

        Self::NV12 { y, uv }
    }

    /// Infer the planes for a full I422 image using the given dimensions
    ///
    /// # Panics
    ///
    /// If `buf` is too small for the given dimensions this function will panic
    #[deny(clippy::arithmetic_side_effects)]
    pub fn infer_i422(buf: S, width: usize, height: usize) -> Self {
        let (y, tmp) = buf.slice_split_at(width.strict_mul_(height));
        let (u, v) = tmp.slice_split_at(width.strict_mul_(height) / 2);

        Self::I422 { y, u, v }
    }

    /// Infer the planes for a full I444 image using the given dimensions
    ///
    /// # Panics
    ///
    /// If `buf` is too small for the given dimensions this function will panic
    #[deny(clippy::arithmetic_side_effects)]
    pub fn infer_i444(buf: S, width: usize, height: usize) -> Self {
        let (y, tmp) = buf.slice_split_at(width.strict_mul_(height));
        let (u, v) = tmp.slice_split_at(width.strict_mul_(height));

        Self::I444 { y, u, v }
    }

    /// Split the planes into multiple planes, where
    /// - `width` is the width of the image which is represented by the planes
    /// - `initial_window` is the window in the image, if the complete image should be processed it should have the same dimensions has the image
    /// - `max_results` how often the image should be split (upper limit, might be less if the image is too small)
    ///
    /// Returns a list containing the new planes and the window piece of the `initial_window`
    ///
    /// # Panics
    ///
    /// Panics when the initial window is larger than the image's dimensions
    #[deny(clippy::arithmetic_side_effects)]
    pub fn split(
        mut self,
        width: usize,
        initial_window: Window,
        max_results: usize,
    ) -> Vec<(Self, Window)> {
        assert!(width >= initial_window.x.strict_add_(initial_window.width));
        assert!(self.bounds_check(
            initial_window.x.strict_add_(initial_window.width),
            initial_window.y.strict_add_(initial_window.height)
        ));

        let mut rects = calculate_windows_by_rows(initial_window, max_results);

        // Ugly hack: Insert a rect as the first window,
        // to have the loop trim the beginning of the planes
        // then remove it in the result
        rects.insert(
            0,
            Window {
                x: 0,
                y: 0,
                width: initial_window.width,
                height: initial_window.y,
            },
        );

        let mut ret = vec![];

        for rect in rects {
            match &mut self {
                Self::I420 { y, u, v } => {
                    let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
                    let (u_, u_remaining) =
                        take(u).slice_split_at(width.strict_mul_(rect.height) / 4);
                    let (v_, v_remaining) =
                        take(v).slice_split_at(width.strict_mul_(rect.height) / 4);

                    *y = y_remaining;
                    *u = u_remaining;
                    *v = v_remaining;

                    ret.push((
                        Self::I420 {
                            y: y_,
                            u: u_,
                            v: v_,
                        },
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::I422 { y, u, v } => {
                    let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
                    let (u_, u_remaining) =
                        take(u).slice_split_at(width.strict_mul_(rect.height) / 2);
                    let (v_, v_remaining) =
                        take(v).slice_split_at(width.strict_mul_(rect.height) / 2);

                    *y = y_remaining;
                    *u = u_remaining;
                    *v = v_remaining;

                    ret.push((
                        Self::I422 {
                            y: y_,
                            u: u_,
                            v: v_,
                        },
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::I444 { y, u, v } => {
                    let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
                    let (u_, u_remaining) = take(u).slice_split_at(width.strict_mul_(rect.height));
                    let (v_, v_remaining) = take(v).slice_split_at(width.strict_mul_(rect.height));

                    *y = y_remaining;
                    *u = u_remaining;
                    *v = v_remaining;

                    ret.push((
                        Self::I444 {
                            y: y_,
                            u: u_,
                            v: v_,
                        },
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::NV12 { y, uv } => {
                    let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
                    let (uv_, uv_remaining) =
                        take(uv).slice_split_at(width.strict_mul_(rect.height) / 2);

                    *y = y_remaining;
                    *uv = uv_remaining;

                    ret.push((
                        Self::NV12 { y: y_, uv: uv_ },
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::YUYV(buf) => {
                    let (x, remaining) =
                        take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(2));
                    *buf = remaining;

                    ret.push((
                        Self::YUYV(x),
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::RGB(buf) => {
                    let (x, remaining) =
                        take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(3));
                    *buf = remaining;
                    ret.push((
                        Self::RGB(x),
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::RGBA(buf) => {
                    let (x, remaining) =
                        take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(4));
                    *buf = remaining;
                    ret.push((
                        Self::RGBA(x),
                        Window {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
            }
        }

        // Part of the before mentioned hack, remove the temporary window
        ret.remove(0);

        ret
    }
}

impl PixelFormatPlanes<&mut [u16]> {
    /// Swap the bytes in the planes to convert between big/little endian
    pub fn swap_bytes(&mut self) {
        use crate::primitive::swap_bytes;

        match self {
            PixelFormatPlanes::I420 { y, u, v }
            | PixelFormatPlanes::I422 { y, u, v }
            | PixelFormatPlanes::I444 { y, u, v } => {
                swap_bytes(y);
                swap_bytes(u);
                swap_bytes(v);
            }
            PixelFormatPlanes::NV12 { y, uv } => {
                swap_bytes(y);
                swap_bytes(uv);
            }
            PixelFormatPlanes::YUYV(buf) => {
                swap_bytes(buf);
            }
            PixelFormatPlanes::RGB(rgb) | PixelFormatPlanes::RGBA(rgb) => {
                swap_bytes(rgb);
            }
        }
    }
}

#[diagnostic::on_unimplemented(message = "AnySlice is only implemented for &[T] and &mut [T].\n\
               When using or Vec<T> or similar try .as_slice() or .as_mut_slice()")]
pub trait AnySlice: sealed::Sealed + Default + Sized {
    fn slice_len(&self) -> usize;
    fn slice_split_at(self, at: usize) -> (Self, Self);
}

mod sealed {
    pub trait Sealed {}
    impl<T> Sealed for &[T] {}
    impl<T> Sealed for &mut [T] {}
}

impl<T> AnySlice for &[T] {
    fn slice_len(&self) -> usize {
        self.len()
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }
}

impl<T> AnySlice for &mut [T] {
    fn slice_len(&self) -> usize {
        self.len()
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at_mut(at)
    }
}

/// Split the work up into windows into the image by calculating the number of rows each thread should handle
fn calculate_windows_by_rows(initial_window: Window, threads: usize) -> Vec<Window> {
    assert_eq!(initial_window.height & 1, 0);

    let sections = initial_window.height / 2;
    let threads = threads.min(sections);

    let parts_per_section = sections / threads;
    let mut remainder = sections % threads;

    let mut rects = Vec::with_capacity(threads);

    for _ in 0..threads {
        let extra = if remainder > 0 {
            remainder -= 1;
            1
        } else {
            0
        };

        let prev = rects.last().unwrap_or(&Window {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });

        rects.push(Window {
            x: initial_window.x,
            y: prev.y + prev.height,
            width: initial_window.width,
            height: (parts_per_section + extra) * 2,
        });
    }

    rects
}

#[cfg(test)]
#[test]
fn verify_windows() {
    let windows = calculate_windows_by_rows(
        Window {
            x: 0,
            y: 0,
            width: 1920,
            height: 1440,
        },
        32,
    );

    let mut prev = windows[0];
    let mut height_accum = prev.height;

    for rect in &windows[1..] {
        assert_eq!(rect.width, 1920);
        assert_eq!(prev.y + prev.height, rect.y);

        height_accum += rect.height;
        prev = *rect;
    }

    assert_eq!(height_accum, 1440);
}
