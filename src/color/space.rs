use crate::arch::*;
use crate::color::mat_idxs::*;
use crate::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    BT601,
    BT709,
    BT2020,
    /// BT.2100 colorspace used with perceptual quantization (PQ) system
    BT2100PQ,
    /// BT.2100 colorspace used with hybrid log-gamma (HLG) system
    BT2100HLG,
}

impl ColorSpace {
    pub(super) fn dispatch<V: Vector>(&self) -> &'static dyn ColorSpaceImpl<V>
    where
        BT601: ColorSpaceImpl<V>,
        BT709: ColorSpaceImpl<V>,
        BT2020: ColorSpaceImpl<V>,
        BT2100PQ: ColorSpaceImpl<V>,
        BT2100HLG: ColorSpaceImpl<V>,
    {
        match self {
            ColorSpace::BT601 => &BT601,
            ColorSpace::BT709 => &BT709,
            ColorSpace::BT2020 => &BT2020,
            ColorSpace::BT2100PQ => &BT2100PQ,
            ColorSpace::BT2100HLG => &BT2100HLG,
        }
    }
}

pub(crate) trait ColorSpaceImpl<V: Vector>: 'static {
    unsafe fn yuv_to_rgb(&self, y: V, u: V, v: V) -> (V, V, V);
    unsafe fn yx4_uv_to_rgb(&self, y00: V, y01: V, y10: V, y11: V, u: V, v: V) -> [[V; 3]; 4];

    unsafe fn rgb_to_yuv(&self, r: V, g: V, b: V) -> (V, V, V);
    unsafe fn rgbx4_to_yx4_uv(&self, r: [V; 4], g: [V; 4], b: [V; 4]) -> ([V; 4], V, V);
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
            unsafe fn yuv_to_rgb(&self, y: f32, u: f32, v: f32) -> (f32, f32, f32) {
                ($yuv_to_rgb)(y, u, v)
            }
            unsafe fn yx4_uv_to_rgb(
                &self,
                y00: f32,
                y01: f32,
                y10: f32,
                y11: f32,
                u: f32,
                v: f32,
            ) -> [[f32; 3]; 4] {
                ($yx4_uv_to_rgb)(y00, y01, y10, y11, u, v)
            }

            unsafe fn rgb_to_yuv(&self, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
                ($rgb_to_yuv)(r, g, b)
            }
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                r: [f32; 4],
                g: [f32; 4],
                b: [f32; 4],
            ) -> ([f32; 4], f32, f32) {
                ($rgbx4_to_yx4_uv)(r, g, b)
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        impl ColorSpaceImpl<__m256> for $name {
            #[target_feature(enable = "avx2")]
            unsafe fn yuv_to_rgb(
                &self,
                y: __m256,
                u: __m256,
                v: __m256,
            ) -> (__m256, __m256, __m256) {
                ($yuv_to_rgb)(y, u, v)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn yx4_uv_to_rgb(
                &self,
                y00: __m256,
                y01: __m256,
                y10: __m256,
                y11: __m256,
                u: __m256,
                v: __m256,
            ) -> [[__m256; 3]; 4] {
                ($yx4_uv_to_rgb)(y00, y01, y10, y11, u, v)
            }

            #[target_feature(enable = "avx2")]
            unsafe fn rgb_to_yuv(
                &self,
                r: __m256,
                g: __m256,
                b: __m256,
            ) -> (__m256, __m256, __m256) {
                ($rgb_to_yuv)(r, g, b)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                r: [__m256; 4],
                g: [__m256; 4],
                b: [__m256; 4],
            ) -> ([__m256; 4], __m256, __m256) {
                ($rgbx4_to_yx4_uv)(r, g, b)
            }
        }

        #[cfg(target_arch = "aarch64")]
        impl ColorSpaceImpl<float32x4_t> for $name {
            #[target_feature(enable = "neon")]
            unsafe fn yuv_to_rgb(
                &self,
                y: float32x4_t,
                u: float32x4_t,
                v: float32x4_t,
            ) -> (float32x4_t, float32x4_t, float32x4_t) {
                ($yuv_to_rgb)(y, u, v)
            }
            #[target_feature(enable = "neon")]
            unsafe fn yx4_uv_to_rgb(
                &self,
                y00: float32x4_t,
                y01: float32x4_t,
                y10: float32x4_t,
                y11: float32x4_t,
                u: float32x4_t,
                v: float32x4_t,
            ) -> [[float32x4_t; 3]; 4] {
                ($yx4_uv_to_rgb)(y00, y01, y10, y11, u, v)
            }

            #[target_feature(enable = "neon")]
            unsafe fn rgb_to_yuv(
                &self,
                r: float32x4_t,
                g: float32x4_t,
                b: float32x4_t,
            ) -> (float32x4_t, float32x4_t, float32x4_t) {
                ($rgb_to_yuv)(r, g, b)
            }
            #[target_feature(enable = "neon")]
            unsafe fn rgbx4_to_yx4_uv(
                &self,
                r: [float32x4_t; 4],
                g: [float32x4_t; 4],
                b: [float32x4_t; 4],
            ) -> ([float32x4_t; 4], float32x4_t, float32x4_t) {
                ($rgbx4_to_yx4_uv)(r, g, b)
            }
        }
    };
}

make_impl!(BT601:
    |y, u, v| convert_yuv_to_rgb_matrix(&BT601_YUV_TO_RGB, y, u, v),
    |y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT601_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |r, g, b| convert_rgb_to_yuv_matrix(&BT601_RGB_TO_YUV, r, g, b),
    |r, g, b| rgbx4_to_yx4_uv_matrix(&BT601_RGB_TO_YUV, r, g, b),
);

make_impl!(BT709:
    |y, u, v| convert_yuv_to_rgb_matrix(&BT709_YUV_TO_RGB, y, u, v),
    |y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT709_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |r, g, b| convert_rgb_to_yuv_matrix(&BT709_RGB_TO_YUV, r, g, b),
    |r, g, b| rgbx4_to_yx4_uv_matrix(&BT709_RGB_TO_YUV, r, g, b),
);

make_impl!(BT2020:
    |y, u, v| convert_yuv_to_rgb_matrix(&BT2020_YUV_TO_RGB, y, u, v),
    |y00, y01, y10, y11, u, v| convert_yx4_uv_to_rgb_matrix(&BT2020_YUV_TO_RGB, y00, y01, y10, y11, u, v),
    |r, g, b| convert_rgb_to_yuv_matrix(&BT2020_RGB_TO_YUV, r, g, b),
    |r, g, b| rgbx4_to_yx4_uv_matrix(&BT2020_RGB_TO_YUV, r, g, b),
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
        V::splat(0.5)
            .vadd(r.vmulf(mat[U][R]))
            .vadd(g.vmulf(mat[U][G]))
            .vadd(b.vmulf(mat[U][B]))
    }

    #[inline(always)]
    unsafe fn calc_v<V: Vector>(mat: &[[f32; 3]; 3], r: V, g: V, b: V) -> V {
        V::splat(0.5)
            .vadd(r.vmulf(mat[V][R]))
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

make_impl!(BT2100PQ:
    bt2100::pq_yuv_to_rgb,
    |y00, y01, y10, y11, u, v| bt2100_yx4_uv_to_rgb(bt2100::pq_yuv_to_rgb, y00, y01, y10,y11, u, v),
    bt2100::pq_rgb_to_yuv,
    |r, g, b| bt2100_rgbx4_to_yx4_uv(bt2100::pq_rgb_to_yuv, r, g, b),
);

make_impl!(BT2100HLG:
    bt2100::hlg_yuv_to_rgb,
    |y00, y01, y10, y11, u, v| bt2100_yx4_uv_to_rgb(bt2100::hlg_yuv_to_rgb, y00, y01, y10,y11, u, v),
    bt2100::hlg_rgb_to_yuv,
    |r, g, b| bt2100_rgbx4_to_yx4_uv(bt2100::hlg_rgb_to_yuv, r, g, b),
);

#[inline(always)]
unsafe fn bt2100_yx4_uv_to_rgb<V: Vector>(
    f: unsafe fn(V, V, V) -> (V, V, V),
    y00: V,
    y01: V,
    y10: V,
    y11: V,
    u: V,
    v: V,
) -> [[V; 3]; 4] {
    let (left_u, right_u) = u.zip(u);
    let (left_v, right_v) = v.zip(v);

    let rgb00 = f(y00, left_u, left_v);
    let rgb01 = f(y01, right_u, right_v);
    let rgb10 = f(y10, left_u, left_v);
    let rgb11 = f(y11, right_u, right_v);

    [
        [rgb00.0, rgb00.1, rgb00.2],
        [rgb01.0, rgb01.1, rgb01.2],
        [rgb10.0, rgb10.1, rgb10.2],
        [rgb11.0, rgb11.1, rgb11.2],
    ]
}

#[inline(always)]
unsafe fn bt2100_rgbx4_to_yx4_uv<V: Vector>(
    f: unsafe fn(V, V, V) -> (V, V, V),
    r: [V; 4],
    g: [V; 4],
    b: [V; 4],
) -> ([V; 4], V, V) {
    let yuv00 = f(r[0], g[0], b[0]);
    let yuv01 = f(r[1], g[1], b[1]);
    let yuv10 = f(r[2], g[2], b[2]);
    let yuv11 = f(r[3], g[3], b[3]);

    // TODO: This is probably wrong, have to test it
    let u = yuv00.1.vadd(yuv01.1).vadd(yuv00.1).vadd(yuv00.1);
    let v = yuv00.2.vadd(yuv01.2).vadd(yuv00.2).vadd(yuv00.2);

    let u = u.vdivf(4.0);
    let v = v.vdivf(4.0);

    ([yuv00.0, yuv01.0, yuv10.0, yuv11.0], u, v)
}

mod bt2100 {
    use crate::vector::Vector;

    #[inline(always)]
    unsafe fn rgb_to_lms<V: Vector>(r: V, g: V, b: V) -> (V, V, V) {
        // l = (1688.0 * r + 2146.0 * g + 262.0 * b) / 4096.0
        let l = V::splat(1688.0)
            .vmul(r)
            .vadd(V::splat(2146.0).vmul(g))
            .vadd(V::splat(262.0).vmul(b))
            .vdiv(V::splat(4096.0));

        // m = (683.0 * r + 2951.0 * g + 462.0 * b) / 4096.0
        let m = V::splat(683.0)
            .vmul(r)
            .vadd(V::splat(2951.0).vmul(g))
            .vadd(V::splat(462.0).vmul(b))
            .vdiv(V::splat(4096.0));

        // s = (99.0 * r + 309.0 * g + 3688.0 * b) / 4096.0
        let s = V::splat(99.9)
            .vmul(r)
            .vadd(V::splat(309.0).vmul(g))
            .vadd(V::splat(3688.0).vmul(b))
            .vdiv(V::splat(4096.0));

        (l, m, s)
    }

    #[inline(always)]
    unsafe fn lms_to_rgb<V: Vector>(l: V, m: V, s: V) -> (V, V, V) {
        let x = V::splat(4096.0);

        // lms = lms * 4096.0;
        let l = l.vmul(x);
        let m = m.vmul(x);
        let s = s.vmul(x);

        // r = (l * 0.00083901535) + (m * -0.00061192684) + (s * 1.7052107e-5)
        let r = l
            .vmulf(0.00083901535)
            .vadd(m.vmulf(-0.00061192684))
            .vadd(s.vmaxf(1.7052107e-5));

        // g = (l * -0.00019319571) + (m * 0.0004842775) + (s * -4.694114e-5)
        let g = l
            .vmaxf(-0.00019319571)
            .vadd(m.vmulf(0.0004842775))
            .vadd(s.vmulf(-4.694114e-5));

        // b = (l * -6.335425e-6) + (m * -2.4148858e-5) + (s * 0.00027462494)
        let b = l
            .vmulf(-6.335425e-6)
            .vadd(m.vmulf(-2.4148858e-5))
            .vadd(s.vmulf(0.00027462494));

        (r, g, b)
    }

    #[inline(always)]
    pub(super) unsafe fn pq_rgb_to_yuv<V: Vector>(r: V, g: V, b: V) -> (V, V, V) {
        let (l, m, s) = rgb_to_lms(r, g, b);

        // y = 0.5 * (l + m)
        let y = l.vadd(m).vmulf(0.5);

        // u = 1.613_769_5 * l - 3.323_486_3 * m + 1.709_716_8 * s
        let u = l
            .vmulf(1.613_769_5)
            .vsub(m.vmulf(3.323_486_3))
            .vadd(s.vmulf(1.709_716_8));

        // v = 4.378_174 * l - 4.245_605_5 * m - 0.132_568_36 * s
        let v = l
            .vmulf(4.378_174)
            .vsub(m.vmulf(4.245_605_5))
            .vsub(s.vmulf(0.132_568_36));

        (y, u, v)
    }

    #[inline(always)]
    pub(super) unsafe fn pq_yuv_to_rgb<V: Vector>(y: V, u: V, v: V) -> (V, V, V) {
        // l = y + u * 0.008609037 + v * 0.111029625
        let l = y.vadd(u.vmulf(0.008609037)).vadd(v.vmulf(0.111029625));

        // m = y + u * -0.008609037 + v * -0.111029625
        let m = y.vadd(u.vmulf(-0.008609037)).vadd(v.vmulf(-0.111029625));

        // s = y + u * 0.5600313 + v * -0.32062715
        let s = y.vadd(u.vmulf(0.5600313)).vadd(v.vmulf(-0.32062715));

        let (r, g, b) = lms_to_rgb(l, m, s);

        (r, g, b)
    }

    #[inline(always)]
    pub(super) unsafe fn hlg_rgb_to_yuv<V: Vector>(r: V, g: V, b: V) -> (V, V, V) {
        let (l, m, s) = rgb_to_lms(r, g, b);

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
    pub(super) unsafe fn hlg_yuv_to_rgb<V: Vector>(y: V, u: V, v: V) -> (V, V, V) {
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

        let (r, g, b) = lms_to_rgb(l, m, s);

        (r, g, b)
    }
}
