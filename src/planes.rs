use crate::{
    plane_decs::{
        PlaneDesc, I01X_PLANES, I21X_PLANES, I41X_PLANES, I420_PLANES, I422_PLANES, I444_PLANES,
        NV12_PLANES,
    },
    util::ArrayIter,
    PixelFormat, StrictApi, Window,
};
use std::mem::MaybeUninit;

#[derive(Debug, thiserror::Error)]
#[error("got invalid number of planes, expected {expected} but only got {got}")]
pub struct InvalidNumberOfPlanesError {
    pub expected: usize,
    pub got: usize,
}

pub(crate) fn read_planes<'a, const N: usize>(
    mut iter: impl Iterator<Item = (&'a [u8], usize)>,
) -> Result<[(&'a [u8], usize); N], InvalidNumberOfPlanesError> {
    let mut out: [(&'a [u8], usize); N] = [(&[], 0); N];

    for (i, out) in out.iter_mut().enumerate() {
        *out = iter.next().ok_or(InvalidNumberOfPlanesError {
            expected: N,
            got: i,
        })?;
    }

    Ok(out)
}

pub(crate) fn read_planes_mut<'a, const N: usize>(
    mut iter: impl Iterator<Item = (&'a mut [u8], usize)>,
) -> Result<[(&'a mut [u8], usize); N], InvalidNumberOfPlanesError> {
    let mut out: [MaybeUninit<(&'a mut [u8], usize)>; N] = [const { MaybeUninit::uninit() }; N];

    for (i, out) in out.iter_mut().enumerate() {
        out.write(iter.next().ok_or(InvalidNumberOfPlanesError {
            expected: N,
            got: i,
        })?);
    }

    Ok(out.map(|plane| unsafe { plane.assume_init() }))
}

/// Infer the planes for an image in the given format using the given dimensions and strides
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer<S: AnySlice>(
    format: PixelFormat,
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> impl Iterator<Item = S> {
    match format {
        PixelFormat::I420 => ArrayIter::from(infer_i420(buf, width, height, strides)),
        PixelFormat::I422 => ArrayIter::from(infer_i422(buf, width, height, strides)),
        PixelFormat::I444 => ArrayIter::from(infer_i444(buf, width, height, strides)),
        PixelFormat::I010 | PixelFormat::I012 => {
            ArrayIter::from(infer_i01x(buf, width, height, strides))
        }
        PixelFormat::I210 | PixelFormat::I212 => {
            ArrayIter::from(infer_i21x(buf, width, height, strides))
        }
        PixelFormat::I410 | PixelFormat::I412 => {
            ArrayIter::from(infer_i41x(buf, width, height, strides))
        }
        PixelFormat::NV12 => ArrayIter::from(infer_nv12(buf, width, height, strides)),
        PixelFormat::YUYV => ArrayIter::from([buf]),
        PixelFormat::RGBA | PixelFormat::BGRA => ArrayIter::from([buf]),
        PixelFormat::RGB | PixelFormat::BGR => ArrayIter::from([buf]),
    }
}

#[deny(clippy::arithmetic_side_effects)]
fn infer_impl<const N: usize, S: AnySlice>(
    plane_decs: [PlaneDesc; N],
    mut buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; N] {
    let strides = strides.map(|strides| <[usize; N]>::try_from(strides).unwrap());

    // Infer default strides for a packed buffer
    let strides: [usize; N] =
        strides.unwrap_or_else(|| plane_decs.map(|desc| desc.packed_stride(width)));

    let mut out: [MaybeUninit<S>; N] = [const { MaybeUninit::uninit() }; N];

    for ((desc, stride), out) in plane_decs.into_iter().zip(strides).zip(out.iter_mut()) {
        let split_at = desc.height_op.op(height).strict_mul_(stride);

        let (prev, rem) = buf.slice_split_at(split_at);

        out.write(prev);
        buf = rem;
    }

    out.map(|p| unsafe { p.assume_init() })
}

/// Infer the planes for a full I420 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i420<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I420_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full I422 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i422<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I422_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full I444 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i444<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I444_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full I010 or I012 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i01x<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I01X_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full I210 or I212 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i21x<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I21X_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full I410 or I412 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_i41x<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 3] {
    infer_impl(I41X_PLANES, buf, width, height, strides)
}

/// Infer the planes for a full NV12 image using the given dimensions
///
/// # Panics
///
/// If `buf` is too small for the given dimensions this function will panic
#[deny(clippy::arithmetic_side_effects)]
pub fn infer_nv12<S: AnySlice>(
    buf: S,
    width: usize,
    height: usize,
    strides: Option<&[usize]>,
) -> [S; 2] {
    infer_impl(NV12_PLANES, buf, width, height, strides)
}

// /// Split the planes into multiple planes, where
// /// - `width` is the width of the image which is represented by the planes
// /// - `initial_window` is the window in the image, if the complete image should be processed it should have the same dimensions has the image
// /// - `max_results` how often the image should be split (upper limit, might be less if the image is too small)
// ///
// /// Returns a list containing the new planes and the window piece of the `initial_window`
// ///
// /// # Panics
// ///
// /// Panics when the initial window is larger than the image's dimensions
// #[deny(clippy::arithmetic_side_effects)]
// pub fn split(
//     planes: ,
//     width: usize,
//     initial_window: Window,
//     max_results: usize,
// ) -> Vec<(Self, Window)> {
//     assert!(width >= initial_window.x.strict_add_(initial_window.width));
//     assert!(self.bounds_check(
//         initial_window.x.strict_add_(initial_window.width),
//         initial_window.y.strict_add_(initial_window.height)
//     ));

//     let mut rects = calculate_windows_by_rows(initial_window, max_results);

//     // Ugly hack: Insert a rect as the first window,
//     // to have the loop trim the beginning of the planes
//     // then remove it in the result
//     rects.insert(
//         0,
//         Window {
//             x: 0,
//             y: 0,
//             width: initial_window.width,
//             height: initial_window.y,
//         },
//     );

//     let mut ret = vec![];

//     for rect in rects {
//         match &mut self {
//             Self::I420 { y, u, v } => {
//                 let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
//                 let (u_, u_remaining) = take(u).slice_split_at(width.strict_mul_(rect.height) / 4);
//                 let (v_, v_remaining) = take(v).slice_split_at(width.strict_mul_(rect.height) / 4);

//                 *y = y_remaining;
//                 *u = u_remaining;
//                 *v = v_remaining;

//                 ret.push((
//                     Self::I420 {
//                         y: y_,
//                         u: u_,
//                         v: v_,
//                     },
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::I422 { y, u, v } => {
//                 let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
//                 let (u_, u_remaining) = take(u).slice_split_at(width.strict_mul_(rect.height) / 2);
//                 let (v_, v_remaining) = take(v).slice_split_at(width.strict_mul_(rect.height) / 2);

//                 *y = y_remaining;
//                 *u = u_remaining;
//                 *v = v_remaining;

//                 ret.push((
//                     Self::I422 {
//                         y: y_,
//                         u: u_,
//                         v: v_,
//                     },
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::I444 { y, u, v } => {
//                 let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
//                 let (u_, u_remaining) = take(u).slice_split_at(width.strict_mul_(rect.height));
//                 let (v_, v_remaining) = take(v).slice_split_at(width.strict_mul_(rect.height));

//                 *y = y_remaining;
//                 *u = u_remaining;
//                 *v = v_remaining;

//                 ret.push((
//                     Self::I444 {
//                         y: y_,
//                         u: u_,
//                         v: v_,
//                     },
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::NV12 { y, uv } => {
//                 let (y_, y_remaining) = take(y).slice_split_at(width.strict_mul_(rect.height));
//                 let (uv_, uv_remaining) =
//                     take(uv).slice_split_at(width.strict_mul_(rect.height) / 2);

//                 *y = y_remaining;
//                 *uv = uv_remaining;

//                 ret.push((
//                     Self::NV12 { y: y_, uv: uv_ },
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::YUYV(buf) => {
//                 let (x, remaining) =
//                     take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(2));
//                 *buf = remaining;

//                 ret.push((
//                     Self::YUYV(x),
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::RGB(buf) => {
//                 let (x, remaining) =
//                     take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(3));
//                 *buf = remaining;
//                 ret.push((
//                     Self::RGB(x),
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//             Self::RGBA(buf) => {
//                 let (x, remaining) =
//                     take(buf).slice_split_at(width.strict_mul_(rect.height).strict_mul_(4));
//                 *buf = remaining;
//                 ret.push((
//                     Self::RGBA(x),
//                     Window {
//                         x: rect.x,
//                         y: 0,
//                         width: rect.width,
//                         height: rect.height,
//                     },
//                 ));
//             }
//         }
//     }

//     // Part of the before mentioned hack, remove the temporary window
//     ret.remove(0);

//     ret
// }

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
    impl<T> Sealed for Vec<T> {}
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

// TODO: remove me
impl<T> AnySlice for Vec<T>
where
    T: Clone,
{
    fn slice_len(&self) -> usize {
        self.len()
    }

    fn slice_split_at(mut self, at: usize) -> (Self, Self) {
        let off = self.split_off(at);

        (self, off)
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
