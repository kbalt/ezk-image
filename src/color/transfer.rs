use crate::vector::Vector;

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
    pub fn linear_to_scaled(&self, mut i: f32) -> f32 {
        // Safety: f32 is not a SIMD type
        unsafe { self.linear_to_scaled_v(&mut [&mut i]) }

        i
    }

    pub fn scaled_to_linear(&self, mut i: f32) -> f32 {
        // Safety: f32 is not a SIMD type
        unsafe { self.scaled_to_linear_v(&mut [&mut i]) }

        i
    }

    #[inline(always)]
    pub(crate) unsafe fn linear_to_scaled_v<const N: usize, V: Vector>(&self, i: &mut [&mut V; N]) {
        match self {
            ColorTransfer::Linear => {}
            ColorTransfer::Gamma22 => {
                for v in i {
                    **v = gamma::linear_to_scaled::<22, V>(**v)
                }
            }
            ColorTransfer::Gamma28 => {
                for v in i {
                    **v = gamma::linear_to_scaled::<28, V>(**v)
                }
            }
            ColorTransfer::SRGB => {
                for v in i {
                    **v = srgb::linear_to_scaled(**v)
                }
            }
            ColorTransfer::SDR => {
                for v in i {
                    **v = sdr::linear_to_scaled(**v)
                }
            }
            ColorTransfer::BT2100PQ => {
                for v in i {
                    **v = bt2100_pq::linear_to_scaled(**v)
                }
            }
            ColorTransfer::BT2100HLG => {
                for v in i {
                    **v = bt2100_hlg::linear_to_scaled(**v)
                }
            }
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn scaled_to_linear_v<const N: usize, V: Vector>(&self, i: &mut [&mut V; N]) {
        match self {
            ColorTransfer::Linear => {}
            ColorTransfer::Gamma22 => {
                for v in i {
                    **v = gamma::scaled_to_linear::<22, V>(**v)
                }
            }
            ColorTransfer::Gamma28 => {
                for v in i {
                    **v = gamma::scaled_to_linear::<28, V>(**v)
                }
            }
            ColorTransfer::SRGB => {
                for v in i {
                    **v = srgb::scaled_to_linear(**v)
                }
            }
            ColorTransfer::SDR => {
                for v in i {
                    **v = sdr::scaled_to_linear(**v)
                }
            }
            ColorTransfer::BT2100PQ => {
                for v in i {
                    **v = bt2100_pq::scaled_to_linear(**v)
                }
            }
            ColorTransfer::BT2100HLG => {
                for v in i {
                    **v = bt2100_hlg::scaled_to_linear(**v)
                }
            }
        }
    }
}

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
        // Avoid producing NaN for negative numbers
        let i = i.vmaxf(0.0);

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
        // Avoid producing NaN for negative numbers
        let i = i.vmaxf(0.0);

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
