use crate::StrictApi as _;
#[cfg(feature = "resize")]
use fir::PixelType::*;

/// Description for a Plane which can be used to implement bounds checks, stride calculation and buffer sizes.
///
/// Not used for the implementation of the format read or write, only utility functions.
#[derive(Clone, Copy)]
pub(crate) struct PlaneDesc {
    pub(crate) width_op: Op,
    pub(crate) height_op: Op,

    /// 1 for u8, 2 for u16
    pub(crate) bytes_per_primitive: usize,

    #[cfg(feature = "resize")]
    pub(crate) pixel_type: fir::PixelType,
}

impl PlaneDesc {
    pub(crate) fn packed_stride(&self, width: usize) -> usize {
        self.width_op
            .op(width)
            .strict_mul_(self.bytes_per_primitive)
    }
}

/// Plane's number of primitives in relation to width / height
#[derive(Clone, Copy)]
pub(crate) enum Op {
    Mul(usize),
    Div(usize),
    Identity,
}

impl Op {
    pub(crate) fn op(self, lhs: usize) -> usize {
        match self {
            Op::Mul(rhs) => lhs.strict_mul_(rhs),
            Op::Div(rhs) => lhs / rhs,
            Op::Identity => lhs,
        }
    }
}

pub(crate) const I420_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
];

pub(crate) const I422_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
];

pub(crate) const I444_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
];

pub(crate) const I01X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
];

pub(crate) const I21X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
];

pub(crate) const I41X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
];

pub(crate) const NV12_PLANES: [PlaneDesc; 2] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Div(2),
        bytes_per_primitive: 1,
        #[cfg(feature = "resize")]
        pixel_type: U8x2,
    },
];

pub(crate) const P01X_PLANES: [PlaneDesc; 2] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Div(2),
        bytes_per_primitive: 2,
        #[cfg(feature = "resize")]
        pixel_type: U16x2,
    },
];

pub(crate) const YUYV_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(2),
    height_op: Op::Identity,
    bytes_per_primitive: 1,
    #[cfg(feature = "resize")]
    pixel_type: U8x4,
}];

pub(crate) const RGBA_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(4),
    height_op: Op::Identity,
    bytes_per_primitive: 1,
    #[cfg(feature = "resize")]
    pixel_type: U8x4,
}];

pub(crate) const RGB_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(3),
    height_op: Op::Identity,
    bytes_per_primitive: 1,
    #[cfg(feature = "resize")]
    pixel_type: U8x3,
}];
