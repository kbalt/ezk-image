#![allow(clippy::too_many_arguments)]

use super::transfer::ColorTransferImpl;
use crate::arch::*;
use crate::color::mat_idxs::*;
use crate::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// YUV Rec. ITU-R BT.601-7 625
    BT601,

    /// YUV Rec. ITU-R BT.709-6
    BT709,

    /// YUV Rec. ITU-R BT.2020-2
    BT2020,

    /// ICtCp Rec. ITU-R BT.2100-2 ICtCp (PQ transfer)
    ICtCpPQ,

    /// ICtCp Rec. ITU-R BT.2100-2 ICtCp (HLG transfer)
    ICtCpHLG,
}

impl ColorSpace {
    pub(super) fn dispatch<V: Vector>(&self) -> &'static dyn ColorSpaceImpl<V>
    where
        BT601: ColorSpaceImpl<V>,
        BT709: ColorSpaceImpl<V>,
        BT2020: ColorSpaceImpl<V>,
        ICtCpPQ: ColorSpaceImpl<V>,
        ICtCpHLG: ColorSpaceImpl<V>,
    {
        match self {
            ColorSpace::BT601 => &BT601,
            ColorSpace::BT709 => &BT709,
            ColorSpace::BT2020 => &BT2020,
            ColorSpace::ICtCpPQ => &ICtCpPQ,
            ColorSpace::ICtCpHLG => &ICtCpHLG,
        }
    }
}

pub(crate) trait ColorSpaceImpl<V: Vector>: 'static {
    unsafe fn yuv_to_rgb(
        &self,
        transfer: &'static dyn ColorTransferImpl<V>,
        xyz_to_rgb: &'static [[f32; 3]; 3],
        y: V,
        u: V,
        v: V,
    ) -> (V, V, V);
    unsafe fn yx4_uv_to_rgb(
        &self,
        transfer: &'static dyn ColorTransferImpl<V>,
        xyz_to_rgb: &'static [[f32; 3]; 3],
        y00: V,
        y01: V,
        y10: V,
        y11: V,
        u: V,
        v: V,
    ) -> [[V; 3]; 4];

    unsafe fn rgb_to_yuv(
        &self,
        transfer: &'static dyn ColorTransferImpl<V>,
        rgb_to_xyz: &'static [[f32; 3]; 3],
        r: V,
        g: V,
        b: V,
    ) -> (V, V, V);
    unsafe fn rgbx4_to_yx4_uv(
        &self,
        transfer: &'static dyn ColorTransferImpl<V>,
        rgb_to_xyz: &'static [[f32; 3]; 3],
        r: [V; 4],
        g: [V; 4],
        b: [V; 4],
    ) -> ([V; 4], V, V);
}

macro_rules! make_impl {
    ($name:ident:
        $yuv_to_rgb:expr,
        $yx4_uv_to_rgb:expr,

        $rgb_to_yuv:expr,
        $rgbx4_to_yx4_uv:expr,
    ) => {
        pub(crate) struct $name;

        impl ColorSpaceImpl<f32> for $name {
            unsafe fn yuv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<f32>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y: f32,
                u: f32,
                v: f32,
            ) -> (f32, f32, f32) {
                ($yuv_to_rgb)(transfer, xyz_to_rgb, y, u, v)
            }
            unsafe fn yx4_uv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<f32>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y00: f32,
                y01: f32,
                y10: f32,
                y11: f32,
                u: f32,
                v: f32,
            ) -> [[f32; 3]; 4] {
                ($yx4_uv_to_rgb)(transfer, xyz_to_rgb, y00, y01, y10, y11, u, v)
            }

            unsafe fn rgb_to_yuv(
                &self,
                transfer: &'static dyn ColorTransferImpl<f32>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: f32,
                g: f32,
                b: f32,
            ) -> (f32, f32, f32) {
                ($rgb_to_yuv)(transfer, rgb_to_xyz, r, g, b)
            }
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                transfer: &'static dyn ColorTransferImpl<f32>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: [f32; 4],
                g: [f32; 4],
                b: [f32; 4],
            ) -> ([f32; 4], f32, f32) {
                ($rgbx4_to_yx4_uv)(transfer, rgb_to_xyz, r, g, b)
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        impl ColorSpaceImpl<__m256> for $name {
            #[target_feature(enable = "avx2")]
            unsafe fn yuv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<__m256>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y: __m256,
                u: __m256,
                v: __m256,
            ) -> (__m256, __m256, __m256) {
                ($yuv_to_rgb)(transfer, xyz_to_rgb, y, u, v)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn yx4_uv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<__m256>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y00: __m256,
                y01: __m256,
                y10: __m256,
                y11: __m256,
                u: __m256,
                v: __m256,
            ) -> [[__m256; 3]; 4] {
                ($yx4_uv_to_rgb)(transfer, xyz_to_rgb, y00, y01, y10, y11, u, v)
            }

            #[target_feature(enable = "avx2")]
            unsafe fn rgb_to_yuv(
                &self,
                transfer: &'static dyn ColorTransferImpl<__m256>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: __m256,
                g: __m256,
                b: __m256,
            ) -> (__m256, __m256, __m256) {
                ($rgb_to_yuv)(transfer, rgb_to_xyz, r, g, b)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                transfer: &'static dyn ColorTransferImpl<__m256>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: [__m256; 4],
                g: [__m256; 4],
                b: [__m256; 4],
            ) -> ([__m256; 4], __m256, __m256) {
                ($rgbx4_to_yx4_uv)(transfer, rgb_to_xyz, r, g, b)
            }
        }

        #[cfg(target_arch = "aarch64")]
        impl ColorSpaceImpl<float32x4_t> for $name {
            #[target_feature(enable = "neon")]
            unsafe fn yuv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<float32x4_t>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y: float32x4_t,
                u: float32x4_t,
                v: float32x4_t,
            ) -> (float32x4_t, float32x4_t, float32x4_t) {
                ($yuv_to_rgb)(transfer, xyz_to_rgb, y, u, v)
            }
            #[target_feature(enable = "neon")]
            unsafe fn yx4_uv_to_rgb(
                &self,
                transfer: &'static dyn ColorTransferImpl<float32x4_t>,
                xyz_to_rgb: &'static [[f32; 3]; 3],
                y00: float32x4_t,
                y01: float32x4_t,
                y10: float32x4_t,
                y11: float32x4_t,
                u: float32x4_t,
                v: float32x4_t,
            ) -> [[float32x4_t; 3]; 4] {
                ($yx4_uv_to_rgb)(transfer, xyz_to_rgb, y00, y01, y10, y11, u, v)
            }

            #[target_feature(enable = "neon")]
            unsafe fn rgb_to_yuv(
                &self,
                transfer: &'static dyn ColorTransferImpl<float32x4_t>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: float32x4_t,
                g: float32x4_t,
                b: float32x4_t,
            ) -> (float32x4_t, float32x4_t, float32x4_t) {
                ($rgb_to_yuv)(transfer, rgb_to_xyz, r, g, b)
            }
            #[target_feature(enable = "neon")]
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                transfer: &'static dyn ColorTransferImpl<float32x4_t>,
                rgb_to_xyz: &'static [[f32; 3]; 3],
                r: [float32x4_t; 4],
                g: [float32x4_t; 4],
                b: [float32x4_t; 4],
            ) -> ([float32x4_t; 4], float32x4_t, float32x4_t) {
                ($rgbx4_to_yx4_uv)(transfer, rgb_to_xyz, r, g, b)
            }
        }
    };
}

make_impl!(BT601:
    |_, _, y, u, v| convert_yuv_to_rgb_matrix(&BT601_YUV_TO_RGB, y, u, v),
    |_, _, y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT601_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |_, _, r, g, b| convert_rgb_to_yuv_matrix(&BT601_RGB_TO_YUV, r, g, b),
    |_, _, r, g, b| rgbx4_to_yx4_uv_matrix(&BT601_RGB_TO_YUV, r, g, b),
);

make_impl!(BT709:
    |_, _, y, u, v| convert_yuv_to_rgb_matrix(&BT709_YUV_TO_RGB, y, u, v),
    |_, _, y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT709_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |_, _, r, g, b| convert_rgb_to_yuv_matrix(&BT709_RGB_TO_YUV, r, g, b),
    |_, _, r, g, b| rgbx4_to_yx4_uv_matrix(&BT709_RGB_TO_YUV, r, g, b),
);

make_impl!(BT2020:
    |_, _, y, u, v| convert_yuv_to_rgb_matrix(&BT2020_YUV_TO_RGB, y, u, v),
    |_, _, y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT2020_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |_, _, r, g, b| convert_rgb_to_yuv_matrix(&BT2020_RGB_TO_YUV, r, g, b),
    |_, _, r, g, b| rgbx4_to_yx4_uv_matrix(&BT2020_RGB_TO_YUV, r, g, b),
);

#[inline(always)]
unsafe fn convert_yuv_to_rgb_matrix<V: Vector>(mat: &[[f32; 3]; 3], y: V, u: V, v: V) -> (V, V, V) {
    let r = y.vadd(v.vmulf(mat[V][R]));
    let g = y.vadd(v.vmulf(mat[V][G]).vadd(u.vmulf(mat[U][G])));
    let b = y.vadd(u.vmulf(mat[U][B]));

    (r, g, b)
}

#[inline(always)]
unsafe fn convert_yx4_uv_to_rgb_matrix<V: Vector>(
    mat: &[[f32; 3]; 3],
    y00: V,
    y01: V,
    y10: V,
    y11: V,
    u: V,
    v: V,
) -> [[V; 3]; 4] {
    #[inline(always)]
    unsafe fn prepare_rgb<V: Vector>(mat: &[[f32; 3]; 3], u: V, v: V) -> (V, V, V) {
        let r = v.vmulf(mat[V][R]);
        let g = v.vmulf(mat[V][G]).vadd(u.vmulf(mat[U][G]));
        let b = u.vmulf(mat[U][B]);

        (r, g, b)
    }

    let (left_u, right_u) = u.zip(u);
    let (left_v, right_v) = v.zip(v);

    let (r_left, g_left, b_left) = prepare_rgb(mat, left_u, left_v);
    let (r_right, g_right, b_right) = prepare_rgb(mat, right_u, right_v);

    let r00 = y00.vadd(r_left);
    let g00 = y00.vadd(g_left);
    let b00 = y00.vadd(b_left);

    let r01 = y01.vadd(r_right);
    let g01 = y01.vadd(g_right);
    let b01 = y01.vadd(b_right);

    let r10 = y10.vadd(r_left);
    let g10 = y10.vadd(g_left);
    let b10 = y10.vadd(b_left);

    let r11 = y11.vadd(r_right);
    let g11 = y11.vadd(g_right);
    let b11 = y11.vadd(b_right);

    [
        [r00, g00, b00],
        [r01, g01, b01],
        [r10, g10, b10],
        [r11, g11, b11],
    ]
}

#[inline(always)]
unsafe fn convert_rgb_to_yuv_matrix<V: Vector>(mat: &[[f32; 3]; 3], r: V, g: V, b: V) -> (V, V, V) {
    let y = r
        .vmulf(mat[0][0])
        .vadd(g.vmulf(mat[0][1]))
        .vadd(b.vmulf(mat[0][2]));
    let u = r
        .vmulf(mat[1][0])
        .vadd(g.vmulf(mat[1][1]))
        .vadd(b.vmulf(mat[1][2]));
    let v = r
        .vmulf(mat[2][0])
        .vadd(g.vmulf(mat[2][1]))
        .vadd(b.vmulf(mat[2][2]));

    (y, u, v)
}

#[inline(always)]
unsafe fn rgbx4_to_yx4_uv_matrix<V: Vector>(
    mat: &[[f32; 3]; 3],
    r: [V; 4],
    g: [V; 4],
    b: [V; 4],
) -> ([V; 4], V, V) {
    #[inline(always)]
    unsafe fn calc_y<V: Vector>(mat: &[[f32; 3]; 3], r: V, g: V, b: V) -> V {
        r.vmulf(mat[Y][R])
            .vadd(g.vmulf(mat[Y][G]))
            .vadd(b.vmulf(mat[Y][B]))
    }

    #[inline(always)]
    unsafe fn calc_u<V: Vector>(mat: &[[f32; 3]; 3], r: V, g: V, b: V) -> V {
        r.vmulf(mat[U][R])
            .vadd(g.vmulf(mat[U][G]))
            .vadd(b.vmulf(mat[U][B]))
    }

    #[inline(always)]
    unsafe fn calc_v<V: Vector>(mat: &[[f32; 3]; 3], r: V, g: V, b: V) -> V {
        r.vmulf(mat[V][R])
            .vadd(g.vmulf(mat[V][G]))
            .vadd(b.vmulf(mat[V][B]))
    }

    // Calculate Y
    let y00 = calc_y(mat, r[0], g[0], b[0]);
    let y01 = calc_y(mat, r[1], g[1], b[1]);
    let y10 = calc_y(mat, r[2], g[2], b[2]);
    let y11 = calc_y(mat, r[3], g[3], b[3]);

    // Calculate U/V pixel from 2x2 rgb pixel blocks
    // Add the upper and lower pixels
    let rgb0_r = r[0].vadd(r[2]);
    let rgb0_g = g[0].vadd(g[2]);
    let rgb0_b = b[0].vadd(b[2]);

    let rgb1_r = r[1].vadd(r[3]);
    let rgb1_g = g[1].vadd(g[3]);
    let rgb1_b = b[1].vadd(b[3]);

    let (rgb0_r, rgb1_r) = rgb0_r.unzip(rgb1_r);
    let (rgb0_g, rgb1_g) = rgb0_g.unzip(rgb1_g);
    let (rgb0_b, rgb1_b) = rgb0_b.unzip(rgb1_b);

    let r = rgb0_r.vadd(rgb1_r);
    let g = rgb0_g.vadd(rgb1_g);
    let b = rgb0_b.vadd(rgb1_b);

    let r = r.vmulf(0.25);
    let g = g.vmulf(0.25);
    let b = b.vmulf(0.25);

    let u = calc_u(mat, r, g, b);
    let v = calc_v(mat, r, g, b);

    ([y00, y01, y10, y11], u, v)
}

#[rustfmt::skip]
macro_rules! make_matrices {
    ($($yuv_to_rgb:ident, $rgb_to_yuv:ident: $kr:expr, $kg:expr, $kb:expr;)*) => {
        $(
        pub(crate) const $yuv_to_rgb: [[f32; 3]; 3] = [
            // R                 G                                  B
            [1.0,                1.0,                               1.0            ], // Y
            [0.0,               (-($kb / $kg)) * (2.0 - 2.0 * $kb), 2.0 - 2.0 * $kb], // U
            [(2.0 - 2.0 * $kr), (-($kr / $kg)) * (2.0 - 2.0 * $kr), 0.0            ], // V
        ];

        pub(crate) const $rgb_to_yuv: [[f32; 3]; 3] = [
            // Y                         U                           V
            [$kr,                        $kg,                        $kb                       ], // R
            [-0.5 * ($kr / (1.0 - $kb)), -0.5 * ($kg / (1.0 - $kb)), 0.5                       ], // G
            [0.5,                        -0.5 * ($kg / (1.0 - $kr)), -0.5 * ($kb / (1.0 - $kr))], // B
        ];
        )*
    };
}

make_matrices! {
    BT601_YUV_TO_RGB, BT601_RGB_TO_YUV: 0.299,  0.587,  0.114;
    BT709_YUV_TO_RGB, BT709_RGB_TO_YUV: 0.2126, 0.7152, 0.0722;
    BT2020_YUV_TO_RGB, BT2020_RGB_TO_YUV: 0.2627, 0.322, 0.0593;
}

make_impl!(ICtCpPQ:
    bt2100::pq_yuv_to_rgb,
    |transfer, xyz_to_rgb, y00, y01, y10, y11, u, v| {
        bt2100_yx4_uv_to_rgb(bt2100::pq_yuv_to_rgb, transfer, xyz_to_rgb, y00, y01, y10,y11, u, v)
    },
    bt2100::pq_rgb_to_yuv,
    |transfer, rgb_to_xyz, r, g, b| {
        bt2100_rgbx4_to_yx4_uv(bt2100::pq_rgb_to_yuv, transfer, rgb_to_xyz, r, g, b)
    },
);

make_impl!(ICtCpHLG:
    bt2100::hlg_yuv_to_rgb,
    |transfer, xyz_to_rgb, y00, y01, y10, y11, u, v| bt2100_yx4_uv_to_rgb(bt2100::hlg_yuv_to_rgb, transfer, xyz_to_rgb, y00, y01, y10,y11, u, v),
    bt2100::hlg_rgb_to_yuv,
    |transfer, rgb_to_xyz, r, g, b| bt2100_rgbx4_to_yx4_uv(bt2100::hlg_rgb_to_yuv, transfer, rgb_to_xyz, r, g, b),
);

#[inline(always)]
unsafe fn bt2100_yx4_uv_to_rgb<V: Vector>(
    f: unsafe fn(&dyn ColorTransferImpl<V>, &[[f32; 3]; 3], V, V, V) -> (V, V, V),
    transfer: &dyn ColorTransferImpl<V>,
    xyz_to_rgb: &[[f32; 3]; 3],
    y00: V,
    y01: V,
    y10: V,
    y11: V,
    u: V,
    v: V,
) -> [[V; 3]; 4] {
    let (left_u, right_u) = u.zip(u);
    let (left_v, right_v) = v.zip(v);

    let rgb00 = f(transfer, xyz_to_rgb, y00, left_u, left_v);
    let rgb01 = f(transfer, xyz_to_rgb, y01, right_u, right_v);
    let rgb10 = f(transfer, xyz_to_rgb, y10, left_u, left_v);
    let rgb11 = f(transfer, xyz_to_rgb, y11, right_u, right_v);

    [
        [rgb00.0, rgb00.1, rgb00.2],
        [rgb01.0, rgb01.1, rgb01.2],
        [rgb10.0, rgb10.1, rgb10.2],
        [rgb11.0, rgb11.1, rgb11.2],
    ]
}

#[inline(always)]
unsafe fn bt2100_rgbx4_to_yx4_uv<V: Vector>(
    f: unsafe fn(&dyn ColorTransferImpl<V>, &[[f32; 3]; 3], V, V, V) -> (V, V, V),
    transfer: &dyn ColorTransferImpl<V>,
    rgb_to_xyz: &[[f32; 3]; 3],
    r: [V; 4],
    g: [V; 4],
    b: [V; 4],
) -> ([V; 4], V, V) {
    let yuv00 = f(transfer, rgb_to_xyz, r[0], g[0], b[0]);
    let yuv01 = f(transfer, rgb_to_xyz, r[1], g[1], b[1]);
    let yuv10 = f(transfer, rgb_to_xyz, r[2], g[2], b[2]);
    let yuv11 = f(transfer, rgb_to_xyz, r[3], g[3], b[3]);

    let u = yuv00.1.vadd(yuv01.1).vadd(yuv00.1).vadd(yuv00.1);
    let v = yuv00.2.vadd(yuv01.2).vadd(yuv00.2).vadd(yuv00.2);

    let u = u.vdivf(4.0);
    let v = v.vdivf(4.0);

    ([yuv00.0, yuv01.0, yuv10.0, yuv11.0], u, v)
}

mod bt2100 {
    use crate::color::primaries::{rgb_to_xyz, xyz_to_rgb};
    use crate::color::transfer::{bt2100_hlg, bt2100_pq, ColorTransferImpl};
    use crate::vector::Vector;

    #[inline(always)]
    unsafe fn xyz_to_lms<V: Vector>(x: V, y: V, z: V) -> (V, V, V) {
        let l = x.vmulf(0.3593).vadd(y.vmulf(0.6976)).vadd(z.vmulf(-0.0359));
        let m = x.vmulf(-0.1921).vadd(y.vmulf(1.1005)).vadd(z.vmulf(0.0754));
        let s = x.vmulf(0.0071).vadd(y.vmulf(0.0748)).vadd(z.vmulf(0.8433));

        (l, m, s)
    }

    #[inline(always)]
    unsafe fn lms_to_xyz<V: Vector>(l: V, m: V, s: V) -> (V, V, V) {
        let x = l
            .vmulf(2.070_034_5)
            .vadd(m.vmulf(-1.326_231_2))
            .vadd(s.vmulf(0.206_702_32));
        let y = l
            .vmulf(0.364_749_8)
            .vadd(m.vmulf(0.680_545_7))
            .vadd(s.vmulf(-0.045_320_32));
        let z = l
            .vmulf(-0.049_781_25)
            .vadd(m.vmulf(-0.049_197_882))
            .vadd(s.vmulf(1.188_097_2));
        (x, y, z)
    }

    #[inline(always)]
    pub(super) unsafe fn pq_rgb_to_yuv<V: Vector>(
        transfer: &dyn ColorTransferImpl<V>,
        rgb_to_xyz_mat: &[[f32; 3]; 3],
        mut r: V,
        mut g: V,
        mut b: V,
    ) -> (V, V, V) {
        transfer.scaled_to_linear3(&mut [&mut r, &mut g, &mut b]);

        let [x, y, z] = rgb_to_xyz(rgb_to_xyz_mat, r, g, b);

        let (l, m, s) = xyz_to_lms(x, y, z);

        let l = bt2100_pq::linear_to_scaled(l);
        let m = bt2100_pq::linear_to_scaled(m);
        let s = bt2100_pq::linear_to_scaled(s);

        // I = 0.5L’ + 0.5M’
        let y = l.vmulf(0.5).vadd(m.vmulf(0.5));

        // Ct = (6610L’ - 13613M’ + 7003S’ ) / 4096
        let u = l
            .vmulf(6610.0)
            .vadd(m.vmulf(-13613.0))
            .vadd(s.vmulf(7003.0))
            .vdivf(4096.0);

        // Cp = (17933L’ - 17390M’ - 543S’ ) / 4096
        let v = l
            .vmulf(17933.0)
            .vadd(m.vmulf(-17390.0))
            .vadd(s.vmulf(-543.0))
            .vdivf(4096.0);

        (y, u, v)
    }

    #[inline(always)]
    pub(super) unsafe fn pq_yuv_to_rgb<V: Vector>(
        transfer: &dyn ColorTransferImpl<V>,
        xyz_to_rgb_mat: &[[f32; 3]; 3],
        y: V,
        u: V,
        v: V,
    ) -> (V, V, V) {
        // l = y + u * 0.008609037 + v * 0.111029625
        let l = y.vadd(u.vmulf(0.008609037)).vadd(v.vmulf(0.111029625));

        // m = y + u * -0.008609037 + v * -0.111029625
        let m = y.vadd(u.vmulf(-0.008609037)).vadd(v.vmulf(-0.111029625));

        // s = y + u * 0.5600313 + v * -0.32062715
        let s = y.vadd(u.vmulf(0.5600313)).vadd(v.vmulf(-0.32062715));

        let l = bt2100_pq::scaled_to_linear(l);
        let m = bt2100_pq::scaled_to_linear(m);
        let s = bt2100_pq::scaled_to_linear(s);

        let (x, y, z) = lms_to_xyz(l, m, s);

        let [mut r, mut g, mut b] = xyz_to_rgb(xyz_to_rgb_mat, x, y, z);

        transfer.linear_to_scaled3(&mut [&mut r, &mut g, &mut b]);

        (r, g, b)
    }

    #[inline(always)]
    pub(super) unsafe fn hlg_rgb_to_yuv<V: Vector>(
        transfer: &dyn ColorTransferImpl<V>,
        rgb_to_xyz_mat: &[[f32; 3]; 3],
        mut r: V,
        mut g: V,
        mut b: V,
    ) -> (V, V, V) {
        transfer.scaled_to_linear3(&mut [&mut r, &mut g, &mut b]);

        let [x, y, z] = rgb_to_xyz(rgb_to_xyz_mat, r, g, b);

        let (l, m, s) = xyz_to_lms(x, y, z);

        let l = bt2100_hlg::linear_to_scaled(l);
        let m = bt2100_hlg::linear_to_scaled(m);
        let s = bt2100_hlg::linear_to_scaled(s);

        // y = 0.5 * (l + m)
        let y = l.vadd(m).vmulf(0.5);

        // u = 0.88500977 * l - 1.8225098 * m + 0.9375 * s
        let u = l
            .vmulf(0.88500977)
            .vsub(m.vmulf(1.8225098))
            .vadd(s.vmulf(0.9375));

        // v = 2.319336 * l - 2.2490234 * m - 0.0703125 * s
        let v = l
            .vmulf(2.319336)
            .vsub(m.vmulf(2.2490234))
            .vsub(s.vmulf(0.0703125));

        (y, u, v)
    }

    #[inline(always)]
    pub(super) unsafe fn hlg_yuv_to_rgb<V: Vector>(
        transfer: &dyn ColorTransferImpl<V>,
        xyz_to_rgb_mat: &[[f32; 3]; 3],
        y: V,
        u: V,
        v: V,
    ) -> (V, V, V) {
        // l = y + u * 0.01571858 + v * 0.20958106
        let l = y
            .vadd(u.vmul(V::splat(0.01571858)))
            .vadd(v.vmul(V::splat(0.20958106)));

        // m = y + u * -0.01571858 + v * -0.20958106
        let m = y
            .vadd(u.vmul(V::splat(-0.01571858)))
            .vadd(v.vmul(V::splat(-0.20958106)));

        // s = y + u * 1.0212711 + v * -0.6052745
        let s = y
            .vadd(u.vmul(V::splat(1.0212711)))
            .vadd(v.vmul(V::splat(-0.6052745)));

        let l = bt2100_hlg::scaled_to_linear(l);
        let m = bt2100_hlg::scaled_to_linear(m);
        let s = bt2100_hlg::scaled_to_linear(s);

        let (x, y, z) = lms_to_xyz(l, m, s);

        let [mut r, mut g, mut b] = xyz_to_rgb(xyz_to_rgb_mat, x, y, z);

        transfer.linear_to_scaled3(&mut [&mut r, &mut g, &mut b]);

        (r, g, b)
    }
}
