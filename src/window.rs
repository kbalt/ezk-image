use std::{error::Error, fmt};

use crate::{
    image::{read_planes, read_planes_mut},
    AnySlice, ColorInfo, ImageMut, ImageRef, PixelFormat,
};

/// Error indicating an invalid [`Window`] for a given image
#[derive(Debug)]
pub struct CropError;

impl fmt::Display for CropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "window position and/or size does not fit in the image")
    }
}

impl Error for CropError {}

/// Cropping window of an [`Image`]
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct Cropped<T>(T, Window);

impl<T: ImageRef> Cropped<T> {
    pub fn new(t: T, window: Window) -> Result<Self, CropError> {
        type Error = CropError;

        if (window.x.checked_add(window.width).ok_or(Error {})? > t.width())
            || (window.y.checked_add(window.height).ok_or(Error {})? > t.height())
        {
            return Err(CropError);
        }

        Ok(Self(t, window))
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: ImageRef> ImageRef for Cropped<T> {
    fn format(&self) -> PixelFormat {
        self.0.format()
    }

    fn width(&self) -> usize {
        self.1.width
    }

    fn height(&self) -> usize {
        self.1.height
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        use PixelFormat::*;

        match self.format() {
            I420 => {
                let [(y, y_stride), (u, u_stride), (v, v_stride)] =
                    read_planes(self.0.planes(), PixelFormat::I420).unwrap();

                trim_i420(self.1, y, u, v, y_stride, u_stride, v_stride)
            }
            I422 => todo!(),
            I444 => todo!(),

            I010 | I012 => todo!(),
            I210 | I212 => todo!(),
            I410 | I412 => todo!(),

            NV12 => todo!(),
            YUYV => todo!(),

            RGBA | BGRA => todo!(),
            RGB | BGR => {
                let [(rgb, stride)] = read_planes(self.0.planes(), PixelFormat::I420).unwrap();

                trim_rgb(self.1, rgb, stride)
            }
        }
    }

    fn color(&self) -> ColorInfo {
        self.0.color()
    }
}

impl<T: ImageMut> ImageMut for Cropped<T> {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        use PixelFormat::*;

        match self.format() {
            I420 => {
                let [(y, y_stride), (u, u_stride), (v, v_stride)] =
                    read_planes_mut(self.0.planes_mut(), PixelFormat::I420).unwrap();

                trim_i420(self.1, y, u, v, y_stride, u_stride, v_stride)
            }
            I422 => todo!(),
            I444 => todo!(),

            I010 | I012 => todo!(),
            I210 | I212 => todo!(),
            I410 | I412 => todo!(),

            NV12 => todo!(),
            YUYV => todo!(),

            RGBA | BGRA => todo!(),
            RGB | BGR => {
                let [(rgb, stride)] =
                    read_planes_mut(self.0.planes_mut(), PixelFormat::I420).unwrap();

                trim_rgb(self.1, rgb, stride)
            }
        }
    }
}

enum SliceMutIter<S> {
    One(Option<S>),
    Two(std::array::IntoIter<S, 2>),
    Thr(std::array::IntoIter<S, 3>),
}

impl<S> Iterator for SliceMutIter<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SliceMutIter::One(opt) => opt.take(),
            SliceMutIter::Two(iter) => iter.next(),
            SliceMutIter::Thr(iter) => iter.next(),
        }
    }
}

fn trim_i420<S: AnySlice>(
    window: Window,
    y: S,
    u: S,
    v: S,
    y_stride: usize,
    u_stride: usize,
    v_stride: usize,
) -> SliceMutIter<(S, usize)> {
    let (_, y) = y.slice_split_at(window.y * y_stride + window.x);
    let (_, u) = u.slice_split_at((window.y / 2) * u_stride + (window.x / 2));
    let (_, v) = v.slice_split_at((window.y / 2) * v_stride + (window.x / 2));

    SliceMutIter::Thr([(y, y_stride), (u, u_stride), (v, v_stride)].into_iter())
}

fn trim_rgb<S: AnySlice>(window: Window, rgb: S, stride: usize) -> SliceMutIter<(S, usize)> {
    let (_, rgb) = rgb.slice_split_at(window.y * stride + window.x * 3);

    SliceMutIter::One(Some((rgb, stride)))
}
