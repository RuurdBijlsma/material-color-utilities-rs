use crate::hct::viewing_conditions::ViewingConditions;
use crate::utils::color_utils::{Argb, ColorUtils, Xyz};
use crate::utils::math_utils::MathUtils;

/// CAM16, a color appearance model. Colors are not just defined by their hex code, but rather, a hex
/// code and viewing conditions.
///
/// CAM16 instances also have coordinates in the CAM16-UCS space, called J*, a*, b*, or jstar, astar,
/// bstar in code. CAM16-UCS is included in the CAM16 specification, and should be used when
/// measuring distances between colors.
///
/// In traditional color spaces, a color can be identified solely by the observer's measurement of
/// the color. Color appearance models such as CAM16 also use information about the environment where
/// the color was observed, known as the viewing conditions.
///
/// For example, white under the traditional assumption of a midday sun white point is accurately
/// measured as a slightly chromatic blue by CAM16. (roughly, hue 203, chroma 3, lightness 100)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cam16 {
    /// Hue in CAM16
    pub hue: f64,
    /// Chroma in CAM16
    pub chroma: f64,
    /// Lightness in CAM16
    pub j: f64,
    /// Brightness in CAM16.
    ///
    /// Prefer lightness, brightness is an absolute quantity. For example, a sheet of white paper is
    /// much brighter viewed in sunlight than in indoor light, but it is the lightest object under any
    /// lighting.
    pub q: f64,
    /// Colorfulness in CAM16.
    ///
    /// Prefer chroma, colorfulness is an absolute quantity. For example, a yellow toy car is much more
    /// colorful outside than inside, but it has the same chroma in both environments.
    pub m: f64,
    /// Saturation in CAM16.
    ///
    /// Colorfulness in proportion to brightness. Prefer chroma, saturation measures colorfulness
    /// relative to the color's own brightness, where chroma is colorfulness relative to white.
    pub s: f64,
    /// Lightness coordinate in CAM16-UCS
    pub jstar: f64,
    /// a* coordinate in CAM16-UCS
    pub astar: f64,
    /// b* coordinate in CAM16-UCS
    pub bstar: f64,
}

impl Cam16 {
    // Transforms XYZ color space coordinates to 'cone'/'RGB' responses in CAM16.
    pub const XYZ_TO_CAM16RGB: [[f64; 3]; 3] = [
        [0.401288, 0.650173, -0.051461],
        [-0.250268, 1.204414, 0.045854],
        [-0.002079, 0.048952, 0.953127],
    ];

    // Transforms 'cone'/'RGB' responses in CAM16 to XYZ color space coordinates.
    pub const CAM16RGB_TO_XYZ: [[f64; 3]; 3] = [
        [1.8620678, -1.0112547, 0.14918678],
        [0.38752654, 0.62144744, -0.00897398],
        [-0.0158415, -0.03412294, 1.0499644],
    ];

    #[must_use]
    pub const fn new(
        hue: f64,
        chroma: f64,
        j: f64,
        q: f64,
        m: f64,
        s: f64,
        jstar: f64,
        astar: f64,
        bstar: f64,
    ) -> Self {
        Self {
            hue,
            chroma,
            j,
            q,
            m,
            s,
            jstar,
            astar,
            bstar,
        }
    }

    /// CAM16 instances also have coordinates in the CAM16-UCS space, called J*, a*, b*, or jstar,
    /// astar, bstar in code. CAM16-UCS is included in the CAM16 specification, and is used to measure
    /// distances between colors.
    #[must_use]
    pub fn distance(&self, other: &Self) -> f64 {
        let d_j = self.jstar - other.jstar;
        let d_a = self.astar - other.astar;
        let d_b = self.bstar - other.bstar;
        let d_e_prime = d_b.mul_add(d_b, d_j.mul_add(d_j, d_a * d_a)).sqrt();
        1.41 * d_e_prime.powf(0.63)
    }

    /// ARGB representation of the color. Assumes the color was viewed in default viewing conditions,
    /// which are near-identical to the default viewing conditions for sRGB.
    #[must_use]
    pub fn to_argb(&self) -> Argb {
        self.viewed(&ViewingConditions::default())
    }

    /// ARGB representation of the color, in defined viewing conditions.
    #[must_use]
    pub fn viewed(&self, viewing_conditions: &ViewingConditions) -> Argb {
        let xyz = self.xyz_in_viewing_conditions(viewing_conditions);
        Argb::from_xyz(xyz)
    }

    #[must_use]
    pub fn xyz_in_viewing_conditions(&self, viewing_conditions: &ViewingConditions) -> Xyz {
        let alpha = if self.chroma == 0.0 || self.j == 0.0 {
            0.0
        } else {
            self.chroma / (self.j / 100.0).sqrt()
        };
        let t = (alpha / (1.64 - 0.29_f64.powf(viewing_conditions.n)).powf(0.73)).powf(1.0 / 0.9);
        let h_rad = self.hue.to_radians();
        let e_hue = 0.25 * ((h_rad + 2.0).cos() + 3.8);
        let ac = viewing_conditions.aw
            * (self.j / 100.0).powf(1.0 / viewing_conditions.c / viewing_conditions.z);
        let p1 = e_hue * (50000.0 / 13.0) * viewing_conditions.nc * viewing_conditions.ncb;
        let p2 = ac / viewing_conditions.nbb;
        let h_sin = h_rad.sin();
        let h_cos = h_rad.cos();
        let gamma = 23.0 * (p2 + 0.305) * t
            / (108.0 * t).mul_add(h_sin, 23.0f64.mul_add(p1, 11.0 * t * h_cos));
        let a = gamma * h_cos;
        let b = gamma * h_sin;
        let r_a = 288.0f64.mul_add(b, 460.0f64.mul_add(p2, 451.0 * a)) / 1403.0;
        let g_a = 261.0f64.mul_add(-b, 460.0f64.mul_add(p2, -(891.0 * a))) / 1403.0;
        let b_a = 6300.0f64.mul_add(-b, 460.0f64.mul_add(p2, -(220.0 * a))) / 1403.0;
        let r_c_base = (27.13 * r_a.abs() / (400.0 - r_a.abs())).max(0.0);
        let r_c = r_a.signum() * (100.0 / viewing_conditions.fl) * r_c_base.powf(1.0 / 0.42);
        let g_c_base = (27.13 * g_a.abs() / (400.0 - g_a.abs())).max(0.0);
        let g_c = g_a.signum() * (100.0 / viewing_conditions.fl) * g_c_base.powf(1.0 / 0.42);
        let b_c_base = (27.13 * b_a.abs() / (400.0 - b_a.abs())).max(0.0);
        let b_c = b_a.signum() * (100.0 / viewing_conditions.fl) * b_c_base.powf(1.0 / 0.42);
        let r_f = r_c / viewing_conditions.rgb_d[0];
        let g_f = g_c / viewing_conditions.rgb_d[1];
        let b_f = b_c / viewing_conditions.rgb_d[2];
        let matrix = Self::CAM16RGB_TO_XYZ;
        let x = r_f * matrix[0][0] + g_f * matrix[0][1] + b_f * matrix[0][2];
        let y = r_f * matrix[1][0] + g_f * matrix[1][1] + b_f * matrix[1][2];
        let z = r_f * matrix[2][0] + g_f * matrix[2][1] + b_f * matrix[2][2];
        Xyz { x, y, z }
    }

    /// Create a CAM16 color from a color, assuming the color was viewed in default viewing
    /// conditions.
    #[must_use]
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_argb_in_viewing_conditions(argb, &ViewingConditions::default())
    }

    /// Create a CAM16 color from a color in defined viewing conditions.
    #[must_use]
    pub fn from_argb_in_viewing_conditions(
        argb: Argb,
        viewing_conditions: &ViewingConditions,
    ) -> Self {
        let red = argb.red();
        let green = argb.green();
        let blue = argb.blue();
        let red_l = ColorUtils::linearized(red);
        let green_l = ColorUtils::linearized(green);
        let blue_l = ColorUtils::linearized(blue);
        let x = 0.18051042f64.mul_add(blue_l, 0.41233895f64.mul_add(red_l, 0.35762064 * green_l));
        let y = 0.0722f64.mul_add(blue_l, 0.2126f64.mul_add(red_l, 0.7152 * green_l));
        let z = 0.95034478f64.mul_add(blue_l, 0.01932141f64.mul_add(red_l, 0.11916382 * green_l));
        Self::from_xyz_in_viewing_conditions(x, y, z, viewing_conditions)
    }

    #[must_use]
    pub fn from_xyz_in_viewing_conditions(
        x: f64,
        y: f64,
        z: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Self {
        let matrix = Self::XYZ_TO_CAM16RGB;
        let r_t = z.mul_add(matrix[0][2], x.mul_add(matrix[0][0], y * matrix[0][1]));
        let g_t = z.mul_add(matrix[1][2], x.mul_add(matrix[1][0], y * matrix[1][1]));
        let b_t = z.mul_add(matrix[2][2], x.mul_add(matrix[2][0], y * matrix[2][1]));

        let r_d = viewing_conditions.rgb_d[0] * r_t;
        let g_d = viewing_conditions.rgb_d[1] * g_t;
        let b_d = viewing_conditions.rgb_d[2] * b_t;

        let r_af = (viewing_conditions.fl * r_d.abs() / 100.0).powf(0.42);
        let g_af = (viewing_conditions.fl * g_d.abs() / 100.0).powf(0.42);
        let b_af = (viewing_conditions.fl * b_d.abs() / 100.0).powf(0.42);
        let r_a = r_d.signum() * 400.0 * r_af / (r_af + 27.13);
        let g_a = g_d.signum() * 400.0 * g_af / (g_af + 27.13);
        let b_a = b_d.signum() * 400.0 * b_af / (b_af + 27.13);

        let a = (11.0f64.mul_add(r_a, -(12.0 * g_a)) + b_a) / 11.0;
        let b = 2.0f64.mul_add(-b_a, r_a + g_a) / 9.0;

        let u = 21.0f64.mul_add(b_a, 20.0f64.mul_add(r_a, 20.0 * g_a)) / 20.0;
        let p2 = (40.0f64.mul_add(r_a, 20.0 * g_a) + b_a) / 20.0;

        let atan2 = b.atan2(a);
        let atan_degrees = atan2.to_degrees();
        let hue = MathUtils::sanitize_degrees_double(atan_degrees);
        let hue_radians = hue.to_radians();

        let ac = p2 * viewing_conditions.nbb;

        let j =
            100.0 * (ac / viewing_conditions.aw).powf(viewing_conditions.c * viewing_conditions.z);
        let q = 4.0 / viewing_conditions.c
            * (j / 100.0).sqrt()
            * (viewing_conditions.aw + 4.0)
            * viewing_conditions.fl_root;

        let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
        let e_hue = 0.25 * ((hue_prime.to_radians() + 2.0).cos() + 3.8);
        let p1 = 50000.0 / 13.0 * e_hue * viewing_conditions.nc * viewing_conditions.ncb;
        let t = p1 * a.hypot(b) / (u + 0.305);
        let alpha = (1.64 - 0.29_f64.powf(viewing_conditions.n)).powf(0.73) * t.powf(0.9);
        let c = alpha * (j / 100.0).sqrt();
        let m = c * viewing_conditions.fl_root;
        let s = 50.0 * (alpha * viewing_conditions.c / (viewing_conditions.aw + 4.0)).sqrt();

        let jstar = 100.0f64.mul_add(0.007, 1.0) * j / 0.007f64.mul_add(j, 1.0);
        let mstar = 1.0 / 0.0228 * (0.0228 * m).ln_1p();
        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();

        Self::new(hue, c, j, q, m, s, jstar, astar, bstar)
    }

    #[must_use]
    pub fn from_jch(j: f64, c: f64, h: f64) -> Self {
        Self::from_jch_in_viewing_conditions(j, c, h, &ViewingConditions::default())
    }

    #[must_use]
    pub fn from_jch_in_viewing_conditions(
        j: f64,
        c: f64,
        h: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Self {
        let q = 4.0 / viewing_conditions.c
            * (j / 100.0).sqrt()
            * (viewing_conditions.aw + 4.0)
            * viewing_conditions.fl_root;
        let m = c * viewing_conditions.fl_root;
        let alpha = c / (j / 100.0).sqrt();
        let s = 50.0 * (alpha * viewing_conditions.c / (viewing_conditions.aw + 4.0)).sqrt();
        let hue_radians = h.to_radians();
        let jstar = 100.0f64.mul_add(0.007, 1.0) * j / 0.007f64.mul_add(j, 1.0);
        let mstar = 1.0 / 0.0228 * (0.0228 * m).ln_1p();
        let astar = mstar * hue_radians.cos();
        let bstar = mstar * hue_radians.sin();
        Self::new(h, c, j, q, m, s, jstar, astar, bstar)
    }

    #[must_use]
    pub fn from_ucs(jstar: f64, astar: f64, bstar: f64) -> Self {
        Self::from_ucs_in_viewing_conditions(jstar, astar, bstar, &ViewingConditions::default())
    }

    #[must_use]
    pub fn from_ucs_in_viewing_conditions(
        jstar: f64,
        astar: f64,
        bstar: f64,
        viewing_conditions: &ViewingConditions,
    ) -> Self {
        let m = astar.hypot(bstar);
        let m2 = (m * 0.0228).exp_m1() / 0.0228;
        let c = m2 / viewing_conditions.fl_root;
        let mut h = bstar.atan2(astar).to_degrees();
        if h < 0.0 {
            h += 360.0;
        }
        let j = jstar / (jstar - 100.0).mul_add(-0.007, 1.0);
        Self::from_jch_in_viewing_conditions(j, c, h, viewing_conditions)
    }
}

// ── Standard conversion traits ──────────────────────────────────────────────

/// sRGB ⇌ CAM16 (default viewing conditions)
impl From<Argb> for Cam16 {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}

impl From<Cam16> for Argb {
    fn from(cam16: Cam16) -> Self {
        cam16.to_argb()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cam16_round_trip() {
        let argb = Argb::from_rgb(255, 0, 0); // Red
        let cam = Cam16::from_argb(argb);
        let argb_back = cam.to_argb();

        // Assert components are close
        assert!((i16::from(argb.red()) - i16::from(argb_back.red())).abs() <= 1);
        assert!((i16::from(argb.green()) - i16::from(argb_back.green())).abs() <= 1);
        assert!((i16::from(argb.blue()) - i16::from(argb_back.blue())).abs() <= 1);
    }

    #[test]
    fn test_cam16_blue() {
        let argb = Argb::from_rgb(0, 0, 255); // Blue
        let cam = Cam16::from_argb(argb);
        // Blue should have a hue around 282 degrees in CAM16
        assert!((cam.hue - 282.78).abs() < 0.1);
    }

    #[test]
    fn test_cam16_ucs_distance() {
        let red = Cam16::from_argb(Argb::from_rgb(255, 0, 0));
        let blue = Cam16::from_argb(Argb::from_rgb(0, 0, 255));
        let dist = red.distance(&blue);
        assert!(dist > 0.0);
        // Distance between Red and Blue in CAM16-UCS is around 21.42
        assert!((dist - 21.42).abs() < 0.1);
    }
}
