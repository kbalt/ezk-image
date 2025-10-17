use crate::{InvalidNumberOfPlanesError, StrictApi as _, plane_decs::*, planes::read_planes};

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// Y, U and V planes, 4:2:0 sub sampling, 8 bits per sample
    I420,

    /// Y, U and V planes, 4:2:2 sub sampling, 8 bits per sample
    I422,

    /// Y, U and V planes, 4:4:4 sub sampling, 8 bits per sample
    I444,

    /// Y, U, and V planes, 4:2:0 sub sampling, 10 bits per sample
    I010,

    /// Y, U, and V planes, 4:2:0 sub sampling, 12 bits per sample
    I012,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    I210,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    I212,

    /// Y, U, and V planes, 4:4:4 sub sampling, 10 bits per sample
    I410,

    /// Y, U, and V planes, 4:4:4 sub sampling, 12 bits per sample
    I412,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 8 bits per sample
    NV12,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 10 bits per sample
    P010,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 12 bits per sample
    P012,

    /// Single YUYV, 4:2:2 sub sampling
    YUYV,

    /// Single RGBA interleaved plane
    RGBA,

    /// Single BGRA interleaved plane
    BGRA,

    /// Single RGB interleaved plane
    RGB,

    /// Single BGR interleaved plane
    BGR,
}

impl PixelFormat {
    /// Calculate the required buffer size given the [`PixelFormat`] self and image dimensions (in pixel width, height).
    ///
    /// The size is the amount of primitives (u8, u16) so when allocating size this must be accounted for.
    #[deny(clippy::arithmetic_side_effects)]
    pub fn buffer_size(self, width: usize, height: usize) -> usize {
        fn buffer_size(planes: &[PlaneDesc], width: usize, height: usize) -> usize {
            let mut size = 0;

            for plane in planes {
                let w = plane.width_op.op(width);
                let h = plane.height_op.op(height);

                size = size.strict_add_(w.strict_mul_(h).strict_mul_(plane.bytes_per_primitive));
            }

            size
        }

        buffer_size(self.plane_desc(), width, height)
    }

    /// Calculate the strides of an image in a packed buffer
    #[deny(clippy::arithmetic_side_effects)]
    pub fn packed_strides(self, width: usize) -> Vec<usize> {
        fn packed_strides(planes: &[PlaneDesc], width: usize) -> Vec<usize> {
            planes
                .iter()
                .map(|desc| desc.packed_stride(width))
                .collect()
        }

        packed_strides(self.plane_desc(), width)
    }

    /// Check if the given planes+strides are valid for dimensions
    #[deny(clippy::arithmetic_side_effects)]
    pub fn bounds_check<'a>(
        self,
        planes: impl Iterator<Item = (&'a [u8], usize)>,
        width: usize,
        height: usize,
    ) -> Result<(), BoundsCheckError> {
        use PixelFormat::*;

        fn bounds_check<const N: usize>(
            planes: [PlaneDesc; N],
            got: [(&[u8], usize); N],
            width: usize,
            height: usize,
        ) -> Result<(), BoundsCheckError> {
            for (i, (plane, (slice, stride))) in planes.into_iter().zip(got).enumerate() {
                // Ensure stride is not smaller than the width would allow
                let min_stride = plane.packed_stride(width);

                if min_stride > stride {
                    return Err(BoundsCheckError::InvalidStride {
                        plane: i,
                        minimum: min_stride,
                        got: stride,
                    });
                }

                // Ensure slice is large enough
                let min_len = stride.strict_mul_(plane.height_op.op(height));

                if min_len > slice.len() {
                    return Err(BoundsCheckError::InvalidPlaneSize {
                        plane: i,
                        minimum: min_len,
                        got: slice.len(),
                    });
                }
            }

            Ok(())
        }

        match self {
            I420 => bounds_check(I420_PLANES, read_planes(planes)?, width, height),
            I422 => bounds_check(I422_PLANES, read_planes(planes)?, width, height),
            I444 => bounds_check(I444_PLANES, read_planes(planes)?, width, height),
            I010 | I012 => bounds_check(I01X_PLANES, read_planes(planes)?, width, height),
            I210 | I212 => bounds_check(I21X_PLANES, read_planes(planes)?, width, height),
            I410 | I412 => bounds_check(I41X_PLANES, read_planes(planes)?, width, height),
            NV12 => bounds_check(NV12_PLANES, read_planes(planes)?, width, height),
            P010 | P012 => bounds_check(P01X_PLANES, read_planes(planes)?, width, height),
            YUYV => bounds_check(YUYV_PLANES, read_planes(planes)?, width, height),
            RGBA | BGRA => bounds_check(RGBA_PLANES, read_planes(planes)?, width, height),
            RGB | BGR => bounds_check(RGB_PLANES, read_planes(planes)?, width, height),
        }
    }

    pub fn bits_per_component(&self) -> usize {
        match self {
            PixelFormat::I420 => 8,
            PixelFormat::I422 => 8,
            PixelFormat::I444 => 8,
            PixelFormat::I010 => 10,
            PixelFormat::I012 => 12,
            PixelFormat::I210 => 10,
            PixelFormat::I212 => 12,
            PixelFormat::I410 => 10,
            PixelFormat::I412 => 12,
            PixelFormat::NV12 => 8,
            PixelFormat::P010 => 10,
            PixelFormat::P012 => 12,
            PixelFormat::YUYV => 8,
            PixelFormat::RGBA => 8,
            PixelFormat::BGRA => 8,
            PixelFormat::RGB => 8,
            PixelFormat::BGR => 8,
        }
    }

    pub(crate) fn plane_desc(&self) -> &'static [PlaneDesc] {
        use PixelFormat::*;

        match self {
            I420 => &I420_PLANES,
            I422 => &I422_PLANES,
            I444 => &I444_PLANES,
            I010 | I012 => &I01X_PLANES,
            I210 | I212 => &I21X_PLANES,
            I410 | I412 => &I41X_PLANES,
            NV12 => &NV12_PLANES,
            P010 | P012 => &P01X_PLANES,
            YUYV => &YUYV_PLANES,
            RGBA | BGRA => &RGBA_PLANES,
            RGB | BGR => &RGB_PLANES,
        }
    }

    pub fn variants() -> impl IntoIterator<Item = Self> {
        use PixelFormat::*;

        [
            I420, I422, I444, I010, I012, I210, I212, I410, I412, NV12, P010, P012, YUYV, RGBA,
            BGRA, RGB, BGR,
        ]
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BoundsCheckError {
    #[error(transparent)]
    InvalidNumberOfPlanes(#[from] InvalidNumberOfPlanesError),

    #[error("invalid stride at plane {plane}, expected it to be at least {minimum}, but got {got}")]
    InvalidStride {
        plane: usize,
        minimum: usize,
        got: usize,
    },

    #[error(
        "invalid plane size at plane {plane}, expected it to be at least {minimum}, but got {got}"
    )]
    InvalidPlaneSize {
        plane: usize,
        minimum: usize,
        got: usize,
    },
}
