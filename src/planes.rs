use crate::{
    PixelFormat, StrictApi,
    plane_decs::{
        I01X_PLANES, I21X_PLANES, I41X_PLANES, I420_PLANES, I422_PLANES, I444_PLANES, NV12_PLANES,
        PlaneDesc,
    },
    util::ArrayIter,
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

/// Helper trait implemented on &[T] and &mut [T]
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
