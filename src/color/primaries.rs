#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    // TODO: remove duplicates
    SRGB,
    BT601PAL,
    BT601NTSC,
    BT709,
    BT2020,
    BT2100,
}

impl ColorPrimaries {
    pub fn rgb_to_xyz_mat(self) -> &'static [[f32; 3]; 3] {
        use ColorPrimaries::*;

        match self {
            SRGB => &generated_consts::SRGB_RGB_TO_XYZ,
            BT601PAL => &generated_consts::BT601PAL_RGB_TO_XYZ,
            BT601NTSC => &generated_consts::BT601NTSC_RGB_TO_XYZ,
            BT709 => &generated_consts::BT709_RGB_TO_XYZ,
            BT2020 => &generated_consts::BT2020_RGB_TO_XYZ,
            BT2100 => &generated_consts::BT2100_RGB_TO_XYZ,
        }
    }

    pub fn xyz_to_rgb_mat(self) -> &'static [[f32; 3]; 3] {
        use ColorPrimaries::*;

        match self {
            SRGB => &generated_consts::SRGB_XYZ_TO_RGB,
            BT601PAL => &generated_consts::BT601PAL_XYZ_TO_RGB,
            BT601NTSC => &generated_consts::BT601NTSC_XYZ_TO_RGB,
            BT709 => &generated_consts::BT709_XYZ_TO_RGB,
            BT2020 => &generated_consts::BT2020_XYZ_TO_RGB,
            BT2100 => &generated_consts::BT2100_XYZ_TO_RGB,
        }
    }
}

mod generated_consts {
    pub(super) const SRGB_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.41239083, 0.21263903, 0.01933082],
        [0.35758436, 0.7151687, 0.11919474],
        [0.1804808, 0.07219231, 0.95053214],
    ];
    pub(super) const SRGB_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [3.2409694, -0.9692435, 0.055630032],
        [-1.537383, 1.8759671, -0.20397685],
        [-0.49861073, 0.04155508, 1.0569714],
    ];

    pub(super) const BT601PAL_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.43055388, 0.22200435, 0.020182215],
        [0.34154972, 0.70665467, 0.12955336],
        [0.17835231, 0.07134092, 0.9393222],
    ];
    pub(super) const BT601PAL_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [3.0633607, -0.9692438, 0.06786106],
        [-1.3933898, 1.8759677, -0.2287993],
        [-0.47582373, 0.041555077, 1.0690897],
    ];

    pub(super) const BT601NTSC_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.39031416, 0.20383073, 0.025401404],
        [0.3700937, 0.71034116, 0.11341577],
        [0.19004808, 0.08582816, 0.95024043],
    ];
    pub(super) const BT601NTSC_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [3.506003, -1.0092703, 0.026740361],
        [-1.7397907, 1.9292052, -0.18375263],
        [-0.5440582, 0.027603257, 1.0636142],
    ];

    pub(super) const BT709_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.41239083, 0.21263903, 0.01933082],
        [0.35758436, 0.7151687, 0.11919474],
        [0.1804808, 0.07219231, 0.95053214],
    ];
    pub(super) const BT709_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [3.2409694, -0.9692435, 0.055630032],
        [-1.537383, 1.8759671, -0.20397685],
        [-0.49861073, 0.04155508, 1.0569714],
    ];

    pub(super) const BT2020_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.63695806, 0.2627002, 0.0],
        [0.14461692, 0.6779981, 0.028072689],
        [0.16888095, 0.05930171, 1.060985],
    ];
    pub(super) const BT2020_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [1.7166512, -0.66668427, 0.017639855],
        [-0.3556708, 1.6164812, -0.04277061],
        [-0.25336626, 0.01576853, 0.9421032],
    ];

    pub(super) const BT2100_RGB_TO_XYZ: [[f32; 3]; 3] = [
        [0.63695806, 0.2627002, 0.0],
        [0.14461692, 0.6779981, 0.028072689],
        [0.16888095, 0.05930171, 1.060985],
    ];
    pub(super) const BT2100_XYZ_TO_RGB: [[f32; 3]; 3] = [
        [1.7166512, -0.66668427, 0.017639855],
        [-0.3556708, 1.6164812, -0.04277061],
        [-0.25336626, 0.01576853, 0.9421032],
    ];
}

#[cfg(test)]
mod generate_matrices {
    use super::ColorPrimaries::{self, *};
    use nalgebra::{Matrix3, Vector3};

    fn xy(x: f32, y: f32) -> Vector3<f32> {
        Vector3::new(x, y, 1.0 - x - y)
    }

    fn xyz_rgbw(primaries: ColorPrimaries) -> [Vector3<f32>; 4] {
        match primaries {
            SRGB | BT709 => [
                xy(0.64, 0.33),
                xy(0.3, 0.6),
                xy(0.15, 0.06),
                xy(0.3127, 0.3290),
            ],
            BT601PAL => [
                xy(0.64, 0.33),
                xy(0.29, 0.6),
                xy(0.15, 0.06),
                xy(0.3127, 0.3290),
            ],
            BT601NTSC => [
                xy(0.63, 0.3290),
                xy(0.31, 0.595),
                xy(0.155, 0.07),
                xy(0.3127, 0.3290),
            ],
            BT2020 | BT2100 => [
                xy(0.708, 0.292),
                xy(0.170, 0.797),
                xy(0.131, 0.046),
                xy(0.3127, 0.3290),
            ],
        }
    }

    fn rgb_to_xyz_mat(primaries: ColorPrimaries) -> Matrix3<f32> {
        let [r, g, b, mut w] = xyz_rgbw(primaries);

        let y = w.y;
        w.x *= 1.0 / y;
        w.y *= 1.0 / y;
        w.z *= 1.0 / y;

        #[rustfmt::skip]
        let m = Matrix3::new(
            r.x, g.x, b.x,
            r.y, g.y, b.y,
            r.z, g.z, b.z
        );

        let s = m.try_inverse().unwrap() * (Vector3::new(w.x, w.y, w.z));

        #[rustfmt::skip]
        let s = Matrix3::new(
            s.x, 0.0, 0.0,
            0.0, s.y, 0.0,
            0.0, 0.0, s.z
        );

        m * s
    }

    #[test]
    #[ignore]
    fn run() {
        let primaries = [SRGB, BT601PAL, BT601NTSC, BT709, BT2020, BT2100];

        for primaries in primaries {
            let rgb_to_xyz = rgb_to_xyz_mat(primaries);
            let xyz_to_rgb = rgb_to_xyz.try_inverse().unwrap();

            println!("pub(super) const {primaries:?}_RGB_TO_XYZ: [[f32; 3]; 3] = {rgb_to_xyz:?};");
            println!("pub(super) const {primaries:?}_XYZ_TO_RGB: [[f32; 3]; 3] = {xyz_to_rgb:?};");
            println!()
        }
    }
}
