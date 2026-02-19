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

use crate::hct::cam16::Cam16;
use crate::utils::color_utils::ColorUtils;
use crate::utils::math_utils::MathUtils;
use std::f64::consts::PI;

/// In traditional color spaces, a color can be identified solely by the observer's measurement of
/// the color. Color appearance models such as CAM16 also use information about the environment where
/// the color was observed, known as the viewing conditions.
///
/// For example, white under the traditional assumption of a midday sun white point is accurately
/// measured as a slightly chromatic blue by CAM16. (roughly, hue 203, chroma 3, lightness 100)
///
/// This class caches intermediate values of the CAM16 conversion process that depend only on viewing
/// conditions, enabling speed ups.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewingConditions {
    pub n: f64,
    pub aw: f64,
    pub nbb: f64,
    pub ncb: f64,
    pub c: f64,
    pub nc: f64,
    pub rgb_d: [f64; 3],
    pub fl: f64,
    pub fl_root: f64,
    pub z: f64,
}

impl ViewingConditions {
    /// Create ViewingConditions from a simple, physically relevant, set of parameters.
    ///
    /// * `white_point`: White point, measured in the XYZ color space. default = D65, or sunny day afternoon
    /// * `adapting_luminance`: The luminance of the adapting field. Informally, how bright it is in
    ///   the room where the color is viewed. Can be calculated from lux by multiplying lux by
    ///   0.0586. default = 11.72, or 200 lux.
    /// * `background_lstar`: The lightness of the area surrounding the color. measured by L* in
    ///   L*a*b*. default = 50.0
    /// * `surround`: A general description of the lighting surrounding the color. 0 is pitch dark,
    ///   like watching a movie in a theater. 1.0 is a dimly light room, like watching TV at home at
    ///   night. 2.0 means there is no difference between the lighting on the color and around it.
    ///   default = 2.0
    /// * `discounting_illuminant`: Whether the eye accounts for the tint of the ambient lighting,
    ///   such as knowing an apple is still red in green light. default = false, the eye does not
    ///   perform this process on self-luminous objects like displays.
    pub fn make(
        white_point: [f64; 3],
        adapting_luminance: f64,
        background_lstar: f64,
        surround: f64,
        discounting_illuminant: bool,
    ) -> Self {
        // A background of pure black is non-physical and leads to infinities that represent the idea
        // that any color viewed in pure black can't be seen.
        let background_lstar = background_lstar.max(0.1);
        // Transform white point XYZ to 'cone'/'rgb' responses
        let matrix = Cam16::XYZ_TO_CAM16RGB;
        let r_w = white_point[0] * matrix[0][0]
            + white_point[1] * matrix[0][1]
            + white_point[2] * matrix[0][2];
        let g_w = white_point[0] * matrix[1][0]
            + white_point[1] * matrix[1][1]
            + white_point[2] * matrix[1][2];
        let b_w = white_point[0] * matrix[2][0]
            + white_point[1] * matrix[2][1]
            + white_point[2] * matrix[2][2];

        let f = 0.8 + surround / 10.0;
        let c = if f >= 0.9 {
            MathUtils::lerp(0.59, 0.69, (f - 0.9) * 10.0)
        } else {
            MathUtils::lerp(0.525, 0.59, (f - 0.8) * 10.0)
        };
        let mut d = if discounting_illuminant {
            1.0
        } else {
            f * (1.0 - (1.0 / 3.6) * ((-adapting_luminance - 42.0) / 92.0).exp())
        };
        d = d.clamp(0.0, 1.0);
        let nc = f;
        let rgb_d = [
            d * (100.0 / r_w) + 1.0 - d,
            d * (100.0 / g_w) + 1.0 - d,
            d * (100.0 / b_w) + 1.0 - d,
        ];
        let k = 1.0 / (5.0 * adapting_luminance + 1.0);
        let k4 = k * k * k * k;
        let k4_f = 1.0 - k4;
        let fl = k4 * adapting_luminance + 0.1 * k4_f * k4_f * (5.0 * adapting_luminance).cbrt();
        let n = ColorUtils::y_from_lstar(background_lstar) / white_point[1];
        let z = 1.48 + n.sqrt();
        let nbb = 0.725 / n.powf(0.2);
        let ncb = nbb;
        let rgb_a_factors = [
            (fl * rgb_d[0] * r_w / 100.0).powf(0.42),
            (fl * rgb_d[1] * g_w / 100.0).powf(0.42),
            (fl * rgb_d[2] * b_w / 100.0).powf(0.42),
        ];
        let rgb_a = [
            400.0 * rgb_a_factors[0] / (rgb_a_factors[0] + 27.13),
            400.0 * rgb_a_factors[1] / (rgb_a_factors[1] + 27.13),
            400.0 * rgb_a_factors[2] / (rgb_a_factors[2] + 27.13),
        ];
        let aw = (2.0 * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
        ViewingConditions {
            n,
            aw,
            nbb,
            ncb,
            c,
            nc,
            rgb_d,
            fl,
            fl_root: fl.powf(0.25),
            z,
        }
    }

    /// Create sRGB-like viewing conditions with a custom background lstar.
    ///
    /// Default viewing conditions have a lstar of 50, midgray.
    pub fn default_with_background_lstar(lstar: f64) -> Self {
        Self::make(
            ColorUtils::white_point_d65(),
            200.0 / PI * ColorUtils::y_from_lstar(50.0) / 100.0,
            lstar,
            2.0,
            false,
        )
    }
}

impl Default for ViewingConditions {
    fn default() -> Self {
        Self::default_with_background_lstar(50.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_viewing_conditions() {
        let vc = ViewingConditions::default();
        // Default n for L* = 50 is roughly 0.18418
        assert!((vc.n - 0.18418).abs() < 0.0001);
        // Default aw is roughly 29.981
        assert!((vc.aw - 29.981).abs() < 0.001);
    }
}
