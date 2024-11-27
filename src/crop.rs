use crate::{
    plane_decs::{
        PlaneDesc, I01X_PLANES, I21X_PLANES, I41X_PLANES, I420_PLANES, I422_PLANES, I444_PLANES,
        NV12_PLANES, RGBA_PLANES, RGB_PLANES, YUYV_PLANES,
    },
    planes::{read_planes, read_planes_mut},
    AnySlice, BoundsCheckError, ColorInfo, ImageMut, ImageRef, ImageRefExt,
    InvalidNumberOfPlanesError, PixelFormat,
};

/// Error indicating an invalid [`Window`] for a given image
#[derive(Debug, thiserror::Error)]
pub enum CropError {
    #[error("the given window coordinates go out of the parent's image bounds")]
    WindowSizeOutOfBounds,

    #[error("The parent image doesn't pass the bounds check: {0}")]
    BoundsCheck(#[from] BoundsCheckError),
}

/// Rect used to mark the "cropping" window
#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Wrapper around [`ImageRef`]/[`ImageMut`] and a [`Window`] cropping the wrapped image
pub struct Cropped<T>(T, Window);

impl<T: ImageRef + ImageRefExt> Cropped<T> {
    pub fn new(t: T, window: Window) -> Result<Self, CropError> {
        t.bounds_check()?;

        let w = window
            .x
            .checked_add(window.width)
            .ok_or(CropError::WindowSizeOutOfBounds)?;

        let h = window
            .y
            .checked_add(window.height)
            .ok_or(CropError::WindowSizeOutOfBounds)?;

        if (w > t.width()) || (h > t.height()) {
            return Err(CropError::WindowSizeOutOfBounds);
        }

        Ok(Self(t, window))
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

unsafe impl<T: ImageRef> ImageRef for Cropped<T> {
    fn format(&self) -> PixelFormat {
        self.0.format()
    }

    fn width(&self) -> usize {
        self.1.width
    }

    fn height(&self) -> usize {
        self.1.height
    }

    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_> {
        use PixelFormat::*;

        match self.format() {
            I420 => crop_planes(I420_PLANES, expect(read_planes(self.0.planes())), self.1),
            I422 => crop_planes(I422_PLANES, expect(read_planes(self.0.planes())), self.1),
            I444 => crop_planes(I444_PLANES, expect(read_planes(self.0.planes())), self.1),

            I010 | I012 => crop_planes(I01X_PLANES, expect(read_planes(self.0.planes())), self.1),
            I210 | I212 => crop_planes(I21X_PLANES, expect(read_planes(self.0.planes())), self.1),
            I410 | I412 => crop_planes(I41X_PLANES, expect(read_planes(self.0.planes())), self.1),

            NV12 => crop_planes(NV12_PLANES, expect(read_planes(self.0.planes())), self.1),
            YUYV => crop_planes(YUYV_PLANES, expect(read_planes(self.0.planes())), self.1),

            RGBA | BGRA => crop_planes(RGBA_PLANES, expect(read_planes(self.0.planes())), self.1),
            RGB | BGR => crop_planes(RGB_PLANES, expect(read_planes(self.0.planes())), self.1),
        }
    }

    fn color(&self) -> ColorInfo {
        self.0.color()
    }
}

unsafe impl<T: ImageMut> ImageMut for Cropped<T> {
    fn planes_mut(&mut self) -> Box<dyn Iterator<Item = (&mut [u8], usize)> + '_> {
        use PixelFormat::*;

        let format = self.format();
        let planes_mut = self.0.planes_mut();

        match format {
            I420 => crop_planes(I420_PLANES, expect(read_planes_mut(planes_mut)), self.1),

            I422 => crop_planes(I422_PLANES, expect(read_planes_mut(planes_mut)), self.1),
            I444 => crop_planes(I444_PLANES, expect(read_planes_mut(planes_mut)), self.1),

            I010 | I012 => crop_planes(I01X_PLANES, expect(read_planes_mut(planes_mut)), self.1),
            I210 | I212 => crop_planes(I21X_PLANES, expect(read_planes_mut(planes_mut)), self.1),
            I410 | I412 => crop_planes(I41X_PLANES, expect(read_planes_mut(planes_mut)), self.1),

            NV12 => crop_planes(NV12_PLANES, expect(read_planes_mut(planes_mut)), self.1),
            YUYV => crop_planes(YUYV_PLANES, expect(read_planes_mut(planes_mut)), self.1),

            RGBA | BGRA => crop_planes(RGBA_PLANES, expect(read_planes_mut(planes_mut)), self.1),
            RGB | BGR => crop_planes(RGB_PLANES, expect(read_planes_mut(planes_mut)), self.1),
        }
    }
}

#[track_caller]
fn expect<T>(result: Result<T, InvalidNumberOfPlanesError>) -> T {
    result.expect("Cropped expects a valid number of planes")
}

fn crop_planes<'s, const N: usize, S: AnySlice + 's>(
    plane_desc: [PlaneDesc; N],
    planes: [(S, usize); N],
    window: Window,
) -> Box<dyn Iterator<Item = (S, usize)> + 's> {
    Box::new(
        plane_desc
            .into_iter()
            .zip(planes)
            .map(move |(plane_desc, (slice, stride))| {
                let x = plane_desc.width_op.op(window.x);
                let y = plane_desc.height_op.op(window.y);

                // First trim the bytes byte "in front" of the window
                let split_at = (y * stride + x) * plane_desc.bytes_per_primitive;
                let (_, slice) = slice.slice_split_at(split_at);

                // Trim the bytes at the end of the window
                let split_at = plane_desc.height_op.op(window.height) * stride;
                let (slice, _) = slice.slice_split_at(split_at);

                (slice, stride)
            }),
    )
}
