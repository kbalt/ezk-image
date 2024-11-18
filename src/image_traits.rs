use crate::{BoundsCheckError, ColorInfo, CropError, Cropped, PixelFormat, Window};

/// # Safety
///
/// Values returned must always be the same every call
pub unsafe trait ImageRef {
    fn format(&self) -> PixelFormat;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    /// Returns an iterator yielding every plane with their associated stride
    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_>;
    fn color(&self) -> ColorInfo;
}

/// # Safety
///
/// Values returned must always be the same every call
pub unsafe trait ImageMut: ImageRef {
    /// Returns an iterator yielding every plane with their associated stride
    fn planes_mut(&mut self) -> Box<dyn Iterator<Item = (&mut [u8], usize)> + '_>;
}

/// [`ImageRef`] extension methods
pub trait ImageRefExt: ImageRef {
    // Perform a bounds check, return an error when it fails
    fn bounds_check(&self) -> Result<(), BoundsCheckError> {
        self.format()
            .bounds_check(self.planes(), self.width(), self.height())
    }

    /// Crop the image with the given window
    fn crop(self, window: Window) -> Result<Cropped<Self>, CropError>
    where
        Self: Sized,
    {
        Cropped::new(self, window)
    }
}

impl<T: ImageRef + ?Sized> ImageRefExt for T {}

unsafe impl<T: ImageRef> ImageRef for &T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

unsafe impl<T: ImageRef> ImageRef for &mut T {
    fn format(&self) -> PixelFormat {
        <T as ImageRef>::format(self)
    }

    fn width(&self) -> usize {
        <T as ImageRef>::width(self)
    }

    fn height(&self) -> usize {
        <T as ImageRef>::height(self)
    }

    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_> {
        <T as ImageRef>::planes(self)
    }

    fn color(&self) -> ColorInfo {
        <T as ImageRef>::color(self)
    }
}

unsafe impl<T: ImageMut> ImageMut for &mut T {
    fn planes_mut(&mut self) -> Box<dyn Iterator<Item = (&mut [u8], usize)> + '_> {
        <T as ImageMut>::planes_mut(self)
    }
}
