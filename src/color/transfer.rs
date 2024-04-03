use crate::arch::*;
use std::convert::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTransfer {
    /// Linear
    Linear,

    /// Gamma of 2.2
    Gamma22,
    /// Gamma of 2.8
    Gamma28,
    /// SRGB (Gamma of 2.2 with linear part)
    SRGB,
    /// BT.601 BT.709 BT.2020
    SDR,
    /// BT.2100 perceptual quantization (PQ) system
    BT2100PQ,
    /// BT.2100 hybrid log-gamma (HLG) system
    BT2100HLG,
}

impl ColorTransfer {
    pub fn linear_to_scaled(&self, i: f32) -> f32 {
        // Safety f32 is safe vector type
        unsafe { self.dispatch().linear_to_scaled(i) }
    }

    pub fn scaled_to_linear(&self, i: f32) -> f32 {
        // Safety f32 is safe vector type
        unsafe { self.dispatch().scaled_to_linear(i) }
    }

    pub(super) fn dispatch<V>(&self) -> &'static dyn ColorTransferImpl<V>
    where
        Linear: ColorTransferImpl<V>,
        Gamma22: ColorTransferImpl<V>,
        Gamma28: ColorTransferImpl<V>,
        Srgb: ColorTransferImpl<V>,
        Sdr: ColorTransferImpl<V>,
        BT2100PQ: ColorTransferImpl<V>,
        BT2100HLG: ColorTransferImpl<V>,
    {
        match self {
            Self::Linear => &Linear,
            Self::Gamma22 => &Gamma22,
            Self::Gamma28 => &Gamma28,
            Self::SRGB => &Srgb,
            Self::SDR => &Sdr,
            Self::BT2100PQ => &BT2100PQ,
            Self::BT2100HLG => &BT2100HLG,
        }
    }
}

pub(crate) trait ColorTransferImpl<V> {
    unsafe fn linear_to_scaled(&self, i: V) -> V;
    unsafe fn linear_to_scaled3(&self, i: &mut [&mut V; 3]);
    unsafe fn linear_to_scaled12(&self, i: &mut [&mut V; 12]);

    unsafe fn scaled_to_linear(&self, i: V) -> V;
    unsafe fn scaled_to_linear3(&self, i: &mut [&mut V; 3]);
    unsafe fn scaled_to_linear12(&self, i: &mut [&mut V; 12]);
}

macro_rules! make_impls {
    ($($name:ident: $linear_to_scaled:expr, $scaled_to_linear:expr;)*) => {
        $(
        pub(crate) struct $name;

        impl ColorTransferImpl<f32> for $name {
            unsafe fn linear_to_scaled(&self, i: f32) -> f32 {
                $linear_to_scaled(i)
            }
            unsafe fn linear_to_scaled3(&self, i: &mut [&mut f32; 3]) {
                for id in 0..3 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }
            unsafe fn linear_to_scaled12(&self, i: &mut [&mut f32; 12]) {
                for id in 0..12 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }

            unsafe fn scaled_to_linear(&self, i: f32) -> f32 {
                $scaled_to_linear(i)
            }
            unsafe fn scaled_to_linear3(&self, i: &mut [&mut f32; 3]) {
                for id in 0..3 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
            unsafe fn scaled_to_linear12(&self, i: &mut [&mut f32; 12]) {
                for id in 0..12 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        impl ColorTransferImpl<__m256> for $name {
            #[target_feature(enable = "avx2")]
            unsafe fn linear_to_scaled(&self, i: __m256) -> __m256 {
                $linear_to_scaled(i)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn linear_to_scaled3(&self, i: &mut [&mut __m256; 3]) {
                for id in 0..3 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }
            #[target_feature(enable = "avx2")]
            unsafe fn linear_to_scaled12(&self, i: &mut [&mut __m256; 12]) {
                for id in 0..12 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }

            #[target_feature(enable = "avx2")]
            unsafe fn scaled_to_linear(&self, i: __m256) -> __m256 {
                $scaled_to_linear(i)
            }
            #[target_feature(enable = "avx2")]
            unsafe fn scaled_to_linear3(&self, i: &mut [&mut __m256; 3]) {
                for id in 0..3 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
            #[target_feature(enable = "avx2")]
            unsafe fn scaled_to_linear12(&self, i: &mut [&mut __m256; 12]) {
                for id in 0..12 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        impl ColorTransferImpl<float32x4_t> for $name {
            #[target_feature(enable = "neon")]
            unsafe fn linear_to_scaled(&self, i: float32x4_t) -> float32x4_t {
                $linear_to_scaled(i)
            }
            #[target_feature(enable = "neon")]
            unsafe fn linear_to_scaled3(&self, i: &mut [&mut float32x4_t; 3]) {
                for id in 0..3 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }
            #[target_feature(enable = "neon")]
            unsafe fn linear_to_scaled12(&self, i: &mut [&mut float32x4_t; 12]) {
                for id in 0..12 {
                    *i[id] = $linear_to_scaled(*i[id]);
                }
            }

            #[target_feature(enable = "neon")]
            unsafe fn scaled_to_linear(&self, i: float32x4_t) -> float32x4_t {
                $scaled_to_linear(i)
            }
            #[target_feature(enable = "neon")]
            unsafe fn scaled_to_linear3(&self, i: &mut [&mut float32x4_t; 3]) {
                for id in 0..3 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
            #[target_feature(enable = "neon")]
            unsafe fn scaled_to_linear12(&self, i: &mut [&mut float32x4_t; 12]) {
                for id in 0..12 {
                    *i[id] = $scaled_to_linear(*i[id]);
                }
            }
        }
        )*

        #[cfg(test)]
        mod self_tests {
            use super::*;

            $(
            #[allow(non_snake_case, unused_unsafe)]
            #[test]
            fn $name() {
                for i in 0..10000 {
                    let i = (i as f32) / 10000.0;

                    let v0 = unsafe { $linear_to_scaled($scaled_to_linear(i)) };
                    let v1 = unsafe { $scaled_to_linear($linear_to_scaled(i)) };

                    assert!((i - v0).abs() < 0.001);
                    assert!((i - v1).abs() < 0.001);
                }
            }
            )*
        }
    };
}

make_impls!(
    Linear:
    identity,
    identity;

    Gamma22:
    gamma::linear_to_scaled::<22, _>,
    gamma::scaled_to_linear::<22, _>;

    Gamma28:
    gamma::linear_to_scaled::<28, _>,
    gamma::scaled_to_linear::<28, _>;

    Srgb:
    srgb::linear_to_scaled,
    srgb::scaled_to_linear;

    Sdr:
    sdr::linear_to_scaled,
    sdr::scaled_to_linear;

    BT2100PQ:
    bt2100_pq::linear_to_scaled,
    bt2100_pq::scaled_to_linear;

    BT2100HLG:
    bt2100_hlg::linear_to_scaled,
    bt2100_hlg::scaled_to_linear;
);

mod gamma {
    use crate::vector::Vector;

    #[inline(always)]
    pub(super) unsafe fn linear_to_scaled<const GAMMA: u32, V: Vector>(i: V) -> V {
        i.vpowf(1.0 / (GAMMA as f32 / 10.0))
    }

    #[inline(always)]
    pub(super) unsafe fn scaled_to_linear<const GAMMA: u32, V: Vector>(i: V) -> V {
        i.vpowf(GAMMA as f32 / 10.0)
    }
}

mod srgb {
    use crate::vector::Vector;

    #[inline(always)]
    pub(super) unsafe fn linear_to_scaled<V: Vector>(i: V) -> V {
        let mask = i.lef(0.0031308);

        // a = i * 12.92
        let a = i.vmulf(12.92);

        // b = 1.055 * i.powf(1.0 / 2.4) - 0.055
        let b = V::splat(1.055).vmul(i.vpowf(1.0 / 2.4)).vsubf(0.055);

        V::select(a, b, mask)
    }

    #[inline(always)]
    pub(super) unsafe fn scaled_to_linear<V: Vector>(i: V) -> V {
        let mask = i.lef(0.04045);

        // a = i / 12.92
        let a = i.vdivf(12.92);

        // b = ((i + 0.055) / 1.055).powf(2.4)
        let b = i.vaddf(0.055).vdivf(1.055).vpowf(2.4);

        V::select(a, b, mask)
    }
}

mod sdr {
    use crate::vector::Vector;

    #[inline(always)]
    pub(crate) unsafe fn linear_to_scaled<V: Vector>(i: V) -> V {
        let mask = i.lt(V::splat(0.018_053_97));

        // a = 4.5 * i
        let a = V::splat(4.5).vmul(i);

        // b = 1.099 * i.powf(0.45) - 0.099
        let b = V::splat(1.099).vmul(i.vpowf(0.45)).vsubf(0.099);

        V::select(a, b, mask)
    }

    #[inline(always)]
    pub(crate) unsafe fn scaled_to_linear<V: Vector>(i: V) -> V {
        let mask = i.ltf(0.081490956);

        // a = i / 4.5
        let a = i.vdivf(4.5);

        // b = ((i + 0.0993) / 1.099).powf(1.0 / 0.45)
        let b = i.vaddf(0.0993).vdivf(1.099).vpowf(1.0 / 0.45);

        V::select(a, b, mask)
    }
}

pub(crate) mod bt2100_pq {
    use crate::vector::Vector;

    const M1: f32 = 0.15930176;
    const M2: f32 = 78.84375;

    const C1: f32 = 0.8359375;
    const C2: f32 = 18.851563;
    const C3: f32 = 18.6875;

    const L: f32 = 10000.0;

    /// PQ inverse EOTF
    #[inline(always)]
    pub(crate) unsafe fn linear_to_scaled<V: Vector>(i: V) -> V {
        let i = i.vdivf(L);
        let ym1 = i.vpowf(M1);

        let a = ym1.vmulf(C2).vaddf(C1);
        let b = ym1.vmulf(C3).vaddf(1.0);

        a.vdiv(b).vpowf(M2)
    }

    /// PQ EOTF
    #[inline(always)]
    pub(crate) unsafe fn scaled_to_linear<V: Vector>(i: V) -> V {
        // Avoid producing NaN for negative numbers
        let i = i.vmaxf(0.0);

        let epow1dm2 = i.vpowf(1.0 / M2);

        let a = epow1dm2.vsubf(C1).vmaxf(0.0);
        let b = V::splat(C2).vsub(epow1dm2.vmulf(C3));

        a.vdiv(b).vpowf(1.0 / M1).vmulf(L)
    }
}

pub(crate) mod bt2100_hlg {
    use crate::vector::Vector;
    use std::f32::consts::E;

    const A: f32 = 0.178_832_77;
    const B: f32 = 0.284_668_92;
    const C: f32 = 0.559_910_7;

    #[inline(always)]
    pub(crate) unsafe fn linear_to_scaled<V: Vector>(i: V) -> V {
        let mask = i.lef(1.0 / 12.0);

        // a = (3.0 * i).sqrt()
        let a = i.vmulf(3.0).vsqrt();

        // b = A * (12.0 * i - B).ln() + C
        let b = V::splat(A)
            .vmul(V::splat(12.0).vmul(i).vsubf(B).vln())
            .vaddf(C);

        V::select(a, b, mask)
    }

    #[inline(always)]
    pub(crate) unsafe fn scaled_to_linear<V: Vector>(i: V) -> V {
        // Avoid producing NaN for negative numbers
        let i = i.vmaxf(0.0);

        let mask = i.lef(0.5);

        // a = i.powf(2.0) / 3.0
        let a = i.vpowf(2.0).vdivf(3.0);

        // b = (E.powf((i - C) / A) + B) / 12.0
        let b = V::splat(E).vpow(i.vsubf(C).vdivf(A)).vaddf(B).vdivf(12.0);

        V::select(a, b, mask)
    }
}
