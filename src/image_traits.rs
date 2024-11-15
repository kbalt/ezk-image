use crate::{BoundsCheckError, ColorInfo, CropError, Cropped, PixelFormat, Window};

pub trait ImageRef {
    fn format(&self) -> PixelFormat;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)>;
    fn color(&self) -> ColorInfo;
}

pub trait ImageMut: ImageRef {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)>;
}

pub trait ImageRefExt: ImageRef {
    fn bounds_check(&self) -> Result<(), BoundsCheckError> {
        self.format()
            .bounds_check(self.planes(), self.width(), self.height())
    }

    fn crop(self, window: Window) -> Result<Cropped<Self>, CropError>
    where
        Self: Sized,
    {
        Cropped::new(self, window)
    }
}

impl<T: ImageRef> ImageRefExt for T {}

impl<T: ImageRef> ImageRef for &T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

impl<T: ImageRef> ImageRef for &mut T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

impl<T: ImageMut> ImageMut for &mut T {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        <T as ImageMut>::planes_mut(self)
    }
}
