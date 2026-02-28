/*
 * Copyright 2025 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use super::math_utils::MathUtils;
use std::fmt;

/// A color in the ARGB color space.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Argb(pub u32);

impl fmt::Debug for Argb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Argb(#{:02X}{:02X}{:02X})",
            self.red(),
            self.green(),
            self.blue()
        )
    }
}

/// A color in the L*a*b* color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

/// A color in the XYZ color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xyz {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Argb {
    const SRGB_TO_XYZ: [[f64; 3]; 3] = [
        [0.41233895, 0.35762064, 0.18051042],
        [0.2126, 0.7152, 0.0722],
        [0.01932141, 0.11916382, 0.95034478],
    ];

    const XYZ_TO_SRGB: [[f64; 3]; 3] = [
        [
            3.2413774792388685,
            -1.5376652402851851,
            -0.49885366846268053,
        ],
        [-0.9691452513005321, 1.8758853451067872, 0.04156585616912061],
        [
            0.05562093689691305,
            -0.20395524564742123,
            1.0571799111220335,
        ],
    ];

    const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

    /// Converts a color from RGB components to ARGB format.
    #[must_use]
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(0xFF000000 | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32))
    }

    /// Converts a color from linear RGB components to ARGB format.
    #[must_use]
    pub fn from_linrgb(linrgb: [f64; 3]) -> Self {
        let r = ColorUtils::delinearized(linrgb[0]);
        let g = ColorUtils::delinearized(linrgb[1]);
        let b = ColorUtils::delinearized(linrgb[2]);
        Self::from_rgb(r, g, b)
    }

    /// Returns the alpha component of a color.
    #[must_use]
    pub const fn alpha(&self) -> u8 {
        ((self.0 >> 24) & 0xFF) as u8
    }

    /// Returns the red component of a color.
    #[must_use]
    pub const fn red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }

    /// Returns the green component of a color.
    #[must_use]
    pub const fn green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    /// Returns the blue component of a color.
    #[must_use]
    pub const fn blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    /// Returns whether a color is opaque.
    #[must_use]
    pub const fn is_opaque(&self) -> bool {
        self.alpha() == 255
    }

    /// Converts a color from XYZ to ARGB.
    #[must_use]
    pub fn from_xyz(xyz: Xyz) -> Self {
        let matrix = Self::XYZ_TO_SRGB;
        let linear_r =
            matrix[0][2].mul_add(xyz.z, matrix[0][0].mul_add(xyz.x, matrix[0][1] * xyz.y));
        let linear_g =
            matrix[1][2].mul_add(xyz.z, matrix[1][0].mul_add(xyz.x, matrix[1][1] * xyz.y));
        let linear_b =
            matrix[2][2].mul_add(xyz.z, matrix[2][0].mul_add(xyz.x, matrix[2][1] * xyz.y));
        let r = ColorUtils::delinearized(linear_r);
        let g = ColorUtils::delinearized(linear_g);
        let b = ColorUtils::delinearized(linear_b);
        Self::from_rgb(r, g, b)
    }

    /// Converts a color from ARGB to XYZ.
    #[must_use]
    pub fn to_xyz(&self) -> Xyz {
        let r = ColorUtils::linearized(self.red());
        let g = ColorUtils::linearized(self.green());
        let b = ColorUtils::linearized(self.blue());
        let result = MathUtils::matrix_multiply([r, g, b], Self::SRGB_TO_XYZ);
        Xyz {
            x: result[0],
            y: result[1],
            z: result[2],
        }
    }

    /// Converts a color from Lab to ARGB.
    #[must_use]
    pub fn from_lab(lab: Lab) -> Self {
        let white_point = Self::WHITE_POINT_D65;
        let fy = (lab.l + 16.0) / 116.0;
        let fx = lab.a / 500.0 + fy;
        let fz = fy - lab.b / 200.0;
        let x_normalized = ColorUtils::lab_invf(fx);
        let y_normalized = ColorUtils::lab_invf(fy);
        let z_normalized = ColorUtils::lab_invf(fz);
        let x = x_normalized * white_point[0];
        let y = y_normalized * white_point[1];
        let z = z_normalized * white_point[2];
        Self::from_xyz(Xyz { x, y, z })
    }

    /// Converts a color from ARGB to Lab.
    #[must_use]
    pub fn to_lab(&self) -> Lab {
        let r = ColorUtils::linearized(self.red());
        let g = ColorUtils::linearized(self.green());
        let b = ColorUtils::linearized(self.blue());
        let matrix = Self::SRGB_TO_XYZ;
        let x = matrix[0][2].mul_add(b, matrix[0][0].mul_add(r, matrix[0][1] * g));
        let y = matrix[1][2].mul_add(b, matrix[1][0].mul_add(r, matrix[1][1] * g));
        let z = matrix[2][2].mul_add(b, matrix[2][0].mul_add(r, matrix[2][1] * g));
        let white_point = Self::WHITE_POINT_D65;
        let x_normalized = x / white_point[0];
        let y_normalized = y / white_point[1];
        let z_normalized = z / white_point[2];
        let fx = ColorUtils::lab_f(x_normalized);
        let fy = ColorUtils::lab_f(y_normalized);
        let fz = ColorUtils::lab_f(z_normalized);
        let l = 116.0f64.mul_add(fy, -16.0);
        let a = 500.0 * (fx - fy);
        let b = 200.0 * (fy - fz);
        Lab { l, a, b }
    }

    /// Converts an L* value to an ARGB representation.
    #[must_use]
    pub fn from_lstar(lstar: f64) -> Self {
        let y = ColorUtils::y_from_lstar(lstar);
        let component = ColorUtils::delinearized(y);
        Self::from_rgb(component, component, component)
    }

    /// Computes the L* value of a color.
    #[must_use]
    pub fn lstar(&self) -> f64 {
        let y = self.to_xyz().y;
        116.0f64.mul_add(ColorUtils::lab_f(y / 100.0), -16.0)
    }
}

/// Color science utilities.
pub struct ColorUtils;

impl ColorUtils {
    /// Linearizes an RGB component.
    #[must_use]
    pub fn linearized(rgb_component: u8) -> f64 {
        let normalized = f64::from(rgb_component) / 255.0;
        if normalized <= 0.040449936 {
            normalized / 12.92 * 100.0
        } else {
            ((normalized + 0.055) / 1.055).powf(2.4) * 100.0
        }
    }

    /// Delinearizes an RGB component.
    #[must_use]
    pub fn delinearized(rgb_component: f64) -> u8 {
        let normalized = rgb_component / 100.0;
        let delinearized: f64 = if normalized <= 0.0031308 {
            normalized * 12.92
        } else {
            1.055f64.mul_add(normalized.powf(1.0 / 2.4), -0.055)
        };
        (delinearized * 255.0).round() as u8
    }

    /// Converts an L* value to a Y value.
    #[must_use]
    pub fn y_from_lstar(lstar: f64) -> f64 {
        100.0 * Self::lab_invf((lstar + 16.0) / 116.0)
    }

    /// Converts a Y value to an L* value.
    #[must_use]
    pub fn lstar_from_y(y: f64) -> f64 {
        Self::lab_f(y / 100.0) * 116.0 - 16.0
    }

    /// Returns the standard white point; white on a sunny day.
    #[must_use]
    pub const fn white_point_d65() -> [f64; 3] {
        [95.047, 100.0, 108.883]
    }

    #[must_use]
    pub fn lab_f(t: f64) -> f64 {
        let e = 216.0 / 24389.0;
        let kappa = 24389.0 / 27.0;
        if t > e {
            t.cbrt()
        } else {
            (kappa * t + 16.0) / 116.0
        }
    }

    #[must_use]
    pub fn lab_invf(ft: f64) -> f64 {
        let e = 216.0 / 24389.0;
        let kappa = 24389.0 / 27.0;
        let ft3 = ft * ft * ft;
        if ft3 > e {
            ft3
        } else {
            116.0f64.mul_add(ft, -16.0) / kappa
        }
    }
}

// ── Standard conversion traits ──────────────────────────────────────────────

/// sRGB ⇌ XYZ
impl From<Argb> for Xyz {
    fn from(argb: Argb) -> Self {
        argb.to_xyz()
    }
}

impl From<Xyz> for Argb {
    fn from(xyz: Xyz) -> Self {
        Self::from_xyz(xyz)
    }
}

/// sRGB ⇌ L*a*b*
impl From<Argb> for Lab {
    fn from(argb: Argb) -> Self {
        argb.to_lab()
    }
}

impl From<Lab> for Argb {
    fn from(lab: Lab) -> Self {
        Argb::from_lab(lab)
    }
}

/// linRGB → sRGB
///
/// Converts a linear-RGB triple `[r, g, b]` (values in 0–100) to `Argb`.
impl From<[f64; 3]> for Argb {
    fn from(linrgb: [f64; 3]) -> Self {
        Argb::from_linrgb(linrgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argb_components() {
        let color = Argb::from_rgb(10, 20, 30);
        assert_eq!(color.red(), 10);
        assert_eq!(color.green(), 20);
        assert_eq!(color.blue(), 30);
        assert_eq!(color.alpha(), 255);
    }

    #[test]
    fn test_is_opaque() {
        let color = Argb::from_rgb(10, 20, 30);
        assert!(color.is_opaque());
        let transparent = Argb(0x00112233);
        assert!(!transparent.is_opaque());
    }

    #[test]
    fn test_linearization_round_trip() {
        for i in 0..=255 {
            let lin = ColorUtils::linearized(i);
            let delin = ColorUtils::delinearized(lin);
            assert_eq!(i, delin);
        }
    }

    #[test]
    fn test_lstar_round_trip() {
        for i in 0..=100 {
            let lstar = f64::from(i);
            let y = ColorUtils::y_from_lstar(lstar);
            let lstar_back = ColorUtils::lstar_from_y(y);
            assert!((lstar - lstar_back).abs() < 1e-10);
        }
    }

    #[test]
    fn test_argb_to_xyz_to_argb() {
        let color = Argb::from_rgb(123, 45, 67);
        let xyz = color.to_xyz();
        let color_back = Argb::from_xyz(xyz);
        assert_eq!(color, color_back);
    }

    #[test]
    fn test_argb_to_lab_to_argb() {
        let color = Argb::from_rgb(123, 45, 67);
        let lab = color.to_lab();
        let color_back = Argb::from_lab(lab);
        assert_eq!(color, color_back);
    }

    #[test]
    fn test_lstar_from_argb() {
        let color = Argb::from_rgb(123, 45, 67);
        let lstar = color.lstar();
        let color_back = Argb::from_lstar(lstar);
        // It's a grayscale converted from lstar, so components should be equal
        assert_eq!(color_back.red(), color_back.green());
        assert_eq!(color_back.green(), color_back.blue());
        // And it should have roughly the same lstar
        assert!((lstar - color_back.lstar()).abs() < 0.1);
    }
}
