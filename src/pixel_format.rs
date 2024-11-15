use crate::{plane_decs::*, planes::read_planes, BoundsCheckError, StrictApi as _};

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

    /// Y and interleaved UV planes, 4:2:0 sub sampling
    NV12,

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
        use PixelFormat::*;

        fn buffer_size<const N: usize>(
            planes: [PlaneDesc; N],
            width: usize,
            height: usize,
        ) -> usize {
            let mut size = 0;

            for plane in planes {
                let w = plane.width_op.op(width);
                let h = plane.height_op.op(height);

                size = size.strict_add_(w.strict_mul_(h).strict_mul_(plane.bytes_per_component));
            }

            size
        }

        match self {
            I420 => buffer_size(I420_PLANES, width, height),
            I422 => buffer_size(I422_PLANES, width, height),
            I444 => buffer_size(I444_PLANES, width, height),
            I010 | I012 => buffer_size(I01X_PLANES, width, height),
            I210 | I212 => buffer_size(I21X_PLANES, width, height),
            I410 | I412 => buffer_size(I41X_PLANES, width, height),
            NV12 => buffer_size(NV12_PLANES, width, height),
            YUYV => buffer_size(YUYV_PLANES, width, height),
            RGBA | BGRA => buffer_size(RGBA_PLANES, width, height),
            RGB | BGR => buffer_size(RGB_PLANES, width, height),
        }
    }

    /// Calculate the strides of an image in a packed buffer
    #[deny(clippy::arithmetic_side_effects)]
    pub fn packed_strides(self, width: usize) -> Vec<usize> {
        use PixelFormat::*;

        fn packed_strides<const N: usize>(planes: [PlaneDesc; N], width: usize) -> Vec<usize> {
            let mut strides = Vec::with_capacity(N);

            for plane in planes {
                strides.push(
                    plane
                        .width_op
                        .op(width)
                        .strict_mul_(plane.bytes_per_component),
                );
            }

            strides
        }

        match self {
            I422 => packed_strides(I422_PLANES, width),
            I420 => packed_strides(I420_PLANES, width),
            I444 => packed_strides(I444_PLANES, width),
            I010 | I012 => packed_strides(I01X_PLANES, width),
            I210 | I212 => packed_strides(I21X_PLANES, width),
            I410 | I412 => packed_strides(I41X_PLANES, width),
            NV12 => packed_strides(NV12_PLANES, width),
            YUYV => packed_strides(YUYV_PLANES, width),
            RGBA | BGRA => packed_strides(RGBA_PLANES, width),
            RGB | BGR => packed_strides(RGB_PLANES, width),
        }
    }

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
                let min_stride = plane
                    .width_op
                    .op(width)
                    .strict_mul_(plane.bytes_per_component);

                if min_stride > stride {
                    return Err(BoundsCheckError::InvalidStride {
                        plane: i,
                        minimum: min_stride,
                        got: stride,
                    });
                }

                // Ensure slice is large enough
                let min_len = stride
                    .strict_mul_(plane.height_op.op(height))
                    .strict_mul_(plane.bytes_per_component);

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
            PixelFormat::YUYV => 8,
            PixelFormat::RGBA => 8,
            PixelFormat::BGRA => 8,
            PixelFormat::RGB => 8,
            PixelFormat::BGR => 8,
        }
    }
}
