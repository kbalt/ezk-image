use crate::StrictApi as _;

pub(crate) struct PlaneDesc {
    pub(crate) width_op: Op,
    pub(crate) height_op: Op,
    pub(crate) bytes_per_component: usize,
}

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
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_component: 1,
    },
];

pub(crate) const I422_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
];

pub(crate) const I444_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
];

pub(crate) const I01X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Div(2),
        bytes_per_component: 2,
    },
];

pub(crate) const I21X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Div(2),
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
];

pub(crate) const I41X_PLANES: [PlaneDesc; 3] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 2,
    },
];

pub(crate) const NV12_PLANES: [PlaneDesc; 2] = [
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Identity,
        bytes_per_component: 1,
    },
    PlaneDesc {
        width_op: Op::Identity,
        height_op: Op::Div(2),
        bytes_per_component: 1,
    },
];

pub(crate) const YUYV_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(2),
    height_op: Op::Identity,
    bytes_per_component: 1,
}];

pub(crate) const RGBA_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(4),
    height_op: Op::Identity,
    bytes_per_component: 1,
}];

pub(crate) const RGB_PLANES: [PlaneDesc; 1] = [PlaneDesc {
    width_op: Op::Mul(3),
    height_op: Op::Identity,
    bytes_per_component: 1,
}];
