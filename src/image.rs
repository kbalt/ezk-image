use crate::{infer, BoundsCheckError, ColorInfo, ImageMut, ImageRef, ImageRefExt, PixelFormat};

/// Basic wrapper around any image, implementing the [`ImageRef`] and [`ImageMut`] trait
#[derive(Debug, Clone)]
pub struct Image<S> {
    format: PixelFormat,
    buffer: Buffer<S>,
    strides: Vec<usize>,
    width: usize,
    height: usize,

    color: ColorInfo,
}

#[derive(Debug, Clone)]
enum Buffer<S> {
    Whole(S),
    Split(Vec<S>),
}

enum BufferIter<T, U, I> {
    Whole(T),
    Split(U),

    _Marker(fn() -> I),
}

impl<T, U, I> Iterator for BufferIter<T, U, I>
where
    T: Iterator<Item = I>,
    U: Iterator<Item = I>,
{
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BufferIter::Whole(iter) => iter.next(),
            BufferIter::Split(iter) => iter.next(),
            BufferIter::_Marker(_) => unreachable!(),
        }
    }
}

/// Everything that can go wrong when constructing an [`Image`]
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("width or height must not be zero")]
    InvalidDimensions,

    #[error(transparent)]
    BoundsCheck(#[from] BoundsCheckError),
}

impl Image<Vec<u8>> {
    pub fn blank(format: PixelFormat, width: usize, height: usize, color: ColorInfo) -> Self {
        Self {
            format,
            buffer: Buffer::Whole(vec![0u8; format.buffer_size(width, height)]),
            strides: format.packed_strides(width),
            width,
            height,
            color,
        }
    }
}

impl<S> Image<S>
where
    Image<S>: ImageRef,
{
    pub fn from_buffer(
        format: PixelFormat,
        buffer: S,
        strides: Option<Vec<usize>>,
        width: usize,
        height: usize,
        color: ColorInfo,
    ) -> Result<Self, ImageError> {
        Self::new(format, Buffer::Whole(buffer), strides, width, height, color)
    }

    pub fn from_planes(
        format: PixelFormat,
        planes: Vec<S>,
        strides: Option<Vec<usize>>,
        width: usize,
        height: usize,
        color: ColorInfo,
    ) -> Result<Self, ImageError> {
        Self::new(format, Buffer::Split(planes), strides, width, height, color)
    }

    fn new(
        format: PixelFormat,
        buffer: Buffer<S>,
        strides: Option<Vec<usize>>,
        width: usize,
        height: usize,
        color: ColorInfo,
    ) -> Result<Self, ImageError> {
        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions);
        }

        let strides = strides.unwrap_or_else(|| format.packed_strides(width));

        let this = Self {
            format,
            buffer,
            strides,
            width,
            height,
            color,
        };

        this.bounds_check()?;

        Ok(this)
    }
}

unsafe impl<S: AsRef<[u8]>> ImageRef for Image<S> {
    fn format(&self) -> PixelFormat {
        self.format
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }

    fn planes(&self) -> impl Iterator<Item = (&[u8], usize)> {
        match &self.buffer {
            Buffer::Whole(buffer) => BufferIter::Whole(
                infer(
                    self.format,
                    buffer.as_ref(),
                    self.width,
                    self.height,
                    Some(&self.strides),
                )
                .zip(self.strides.iter().copied()),
            ),
            Buffer::Split(planes) => BufferIter::Split(
                planes
                    .iter()
                    .map(|p| p.as_ref())
                    .zip(self.strides.iter().copied()),
            ),
        }
    }
    fn color(&self) -> ColorInfo {
        self.color
    }
}

unsafe impl<S: AsRef<[u8]> + AsMut<[u8]>> ImageMut for Image<S> {
    fn planes_mut(&mut self) -> impl Iterator<Item = (&mut [u8], usize)> {
        match &mut self.buffer {
            Buffer::Whole(buffer) => BufferIter::Whole(
                infer(
                    self.format,
                    buffer.as_mut(),
                    self.width,
                    self.height,
                    Some(&self.strides),
                )
                .zip(self.strides.iter().copied()),
            ),
            Buffer::Split(planes) => BufferIter::Split(
                planes
                    .iter_mut()
                    .map(|plane| plane.as_mut())
                    .zip(self.strides.iter().copied()),
            ),
        }
    }
}
