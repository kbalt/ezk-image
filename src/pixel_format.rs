use crate::{InvalidNumberOfPlanesError, StrictApi as _, plane_decs::*, planes::read_planes};

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// Y, U and V planes, 4:2:0 sub sampling, 8 bits per sample
    #[cfg(feature = "I420")]
    I420,

    /// Y, U and V planes, 4:2:2 sub sampling, 8 bits per sample
    #[cfg(feature = "I422")]
    I422,

    /// Y, U and V planes, 4:4:4 sub sampling, 8 bits per sample
    #[cfg(feature = "I444")]
    I444,

    /// Y, U, and V planes, 4:2:0 sub sampling, 10 bits per sample
    #[cfg(feature = "I010")]
    I010,

    /// Y, U, and V planes, 4:2:0 sub sampling, 12 bits per sample
    #[cfg(feature = "I012")]
    I012,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    #[cfg(feature = "I210")]
    I210,

    /// Y, U, and V planes, 4:2:2 sub sampling, 10 bits per sample
    #[cfg(feature = "I212")]
    I212,

    /// Y, U, and V planes, 4:4:4 sub sampling, 10 bits per sample
    #[cfg(feature = "I410")]
    I410,

    /// Y, U, and V planes, 4:4:4 sub sampling, 12 bits per sample
    #[cfg(feature = "I412")]
    I412,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 8 bits per sample
    #[cfg(feature = "NV12")]
    NV12,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 10 bits per sample
    #[cfg(feature = "P010")]
    P010,

    /// Y and interleaved UV planes, 4:2:0 sub sampling, 12 bits per sample
    #[cfg(feature = "P012")]
    P012,

    /// Single YUYV, 4:2:2 sub sampling
    #[cfg(feature = "YUYV")]
    YUYV,

    /// Single RGBA interleaved plane
    #[cfg(feature = "RGBA")]
    RGBA,

    /// Single BGRA interleaved plane
    #[cfg(feature = "BGRA")]
    BGRA,

    /// Single ARGB interleaved plane
    #[cfg(feature = "ARGB")]
    ARGB,

    /// Single ABGR interleaved plane
    #[cfg(feature = "ABGR")]
    ABGR,

    /// Single RGB interleaved plane
    #[cfg(feature = "RGB")]
    RGB,

    /// Single BGR interleaved plane
    #[cfg(feature = "BGR")]
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
            #[cfg(feature = "I420")]
            I420 => bounds_check(I420_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I422")]
            I422 => bounds_check(I422_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I444")]
            I444 => bounds_check(I444_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I010")]
            I010 => bounds_check(I01X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I012")]
            I012 => bounds_check(I01X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I210")]
            I210 => bounds_check(I21X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I212")]
            I212 => bounds_check(I21X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I410")]
            I410 => bounds_check(I41X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "I412")]
            I412 => bounds_check(I41X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "NV12")]
            NV12 => bounds_check(NV12_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "P010")]
            P010 => bounds_check(P01X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "P012")]
            P012 => bounds_check(P01X_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "YUYV")]
            YUYV => bounds_check(YUYV_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "RGBA")]
            RGBA => bounds_check(RGBA_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "BGRA")]
            BGRA => bounds_check(RGBA_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "ARGB")]
            ARGB => bounds_check(RGBA_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "ABGR")]
            ABGR => bounds_check(RGBA_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "RGB")]
            RGB => bounds_check(RGB_PLANES, read_planes(planes)?, width, height),
            #[cfg(feature = "BGR")]
            BGR => bounds_check(RGB_PLANES, read_planes(planes)?, width, height),
        }
    }

    pub fn bits_per_component(&self) -> usize {
        match self {
            #[cfg(feature = "I420")]
            PixelFormat::I420 => 8,
            #[cfg(feature = "I422")]
            PixelFormat::I422 => 8,
            #[cfg(feature = "I444")]
            PixelFormat::I444 => 8,
            #[cfg(feature = "I010")]
            PixelFormat::I010 => 10,
            #[cfg(feature = "I012")]
            PixelFormat::I012 => 12,
            #[cfg(feature = "I210")]
            PixelFormat::I210 => 10,
            #[cfg(feature = "I212")]
            PixelFormat::I212 => 12,
            #[cfg(feature = "I410")]
            PixelFormat::I410 => 10,
            #[cfg(feature = "I412")]
            PixelFormat::I412 => 12,
            #[cfg(feature = "NV12")]
            PixelFormat::NV12 => 8,
            #[cfg(feature = "P010")]
            PixelFormat::P010 => 10,
            #[cfg(feature = "P012")]
            PixelFormat::P012 => 12,
            #[cfg(feature = "YUYV")]
            PixelFormat::YUYV => 8,
            #[cfg(feature = "RGBA")]
            PixelFormat::RGBA => 8,
            #[cfg(feature = "BGRA")]
            PixelFormat::BGRA => 8,
            #[cfg(feature = "ARGB")]
            PixelFormat::ARGB => 8,
            #[cfg(feature = "ABGR")]
            PixelFormat::ABGR => 8,
            #[cfg(feature = "RGB")]
            PixelFormat::RGB => 8,
            #[cfg(feature = "BGR")]
            PixelFormat::BGR => 8,
        }
    }

    pub(crate) fn plane_desc(&self) -> &'static [PlaneDesc] {
        use PixelFormat::*;

        match self {
            #[cfg(feature = "I420")]
            I420 => &I420_PLANES,
            #[cfg(feature = "I422")]
            I422 => &I422_PLANES,
            #[cfg(feature = "I444")]
            I444 => &I444_PLANES,
            #[cfg(feature = "I010")]
            I010 => &I01X_PLANES,
            #[cfg(feature = "I012")]
            I012 => &I01X_PLANES,
            #[cfg(feature = "I210")]
            I210 => &I21X_PLANES,
            #[cfg(feature = "I212")]
            I212 => &I21X_PLANES,
            #[cfg(feature = "I410")]
            I410 => &I41X_PLANES,
            #[cfg(feature = "I412")]
            I412 => &I41X_PLANES,
            #[cfg(feature = "NV12")]
            NV12 => &NV12_PLANES,
            #[cfg(feature = "P010")]
            P010 => &P01X_PLANES,
            #[cfg(feature = "P012")]
            P012 => &P01X_PLANES,
            #[cfg(feature = "YUYV")]
            YUYV => &YUYV_PLANES,
            #[cfg(feature = "RGBA")]
            RGBA => &RGBA_PLANES,
            #[cfg(feature = "BGRA")]
            BGRA => &RGBA_PLANES,
            #[cfg(feature = "ARGB")]
            ARGB => &RGBA_PLANES,
            #[cfg(feature = "ABGR")]
            ABGR => &RGBA_PLANES,
            #[cfg(feature = "RGB")]
            RGB => &RGB_PLANES,
            #[cfg(feature = "BGR")]
            BGR => &RGB_PLANES,
        }
    }

    pub fn variants() -> impl IntoIterator<Item = Self> {
        use PixelFormat::*;

        [
            #[cfg(feature = "I420")]
            I420,
            #[cfg(feature = "I422")]
            I422,
            #[cfg(feature = "I444")]
            I444,
            #[cfg(feature = "I010")]
            I010,
            #[cfg(feature = "I012")]
            I012,
            #[cfg(feature = "I210")]
            I210,
            #[cfg(feature = "I212")]
            I212,
            #[cfg(feature = "I410")]
            I410,
            #[cfg(feature = "I412")]
            I412,
            #[cfg(feature = "NV12")]
            NV12,
            #[cfg(feature = "P010")]
            P010,
            #[cfg(feature = "P012")]
            P012,
            #[cfg(feature = "YUYV")]
            YUYV,
            #[cfg(feature = "RGBA")]
            RGBA,
            #[cfg(feature = "BGRA")]
            BGRA,
            #[cfg(feature = "ARGB")]
            ARGB,
            #[cfg(feature = "ABGR")]
            ABGR,
            #[cfg(feature = "RGB")]
            RGB,
            #[cfg(feature = "BGR")]
            BGR,
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
