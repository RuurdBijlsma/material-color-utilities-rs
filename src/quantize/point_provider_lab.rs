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

use crate::quantize::point_provider::PointProvider;
use crate::utils::color_utils::{Argb, Lab};

/// Provides conversions needed for K-Means quantization. Converting input to points, and converting
/// the final state of the K-Means algorithm to colors.
pub struct PointProviderLab;

impl PointProvider for PointProviderLab {
    /// Convert a color represented in ARGB to a 3-element array of L*a*b* coordinates of the color.
    fn from_argb(argb: Argb) -> [f64; 3] {
        let lab = argb.to_lab();
        [lab.l, lab.a, lab.b]
    }

    /// Convert a 3-element array to a color represented in ARGB.
    fn to_argb(point: [f64; 3]) -> Argb {
        Argb::from_lab(Lab {
            l: point[0],
            a: point[1],
            b: point[2],
        })
    }

    /// Standard CIE 1976 delta E formula also takes the square root, unneeded here. This method is
    /// used by quantization algorithms to compare distance, and the relative ordering is the same,
    /// with or without a square root.
    ///
    /// This relatively minor optimization is helpful because this method is called at least once for
    /// each pixel in an image.
    fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
        let d_l = a[0] - b[0];
        let d_a = a[1] - b[1];
        let d_b = a[2] - b[2];
        d_l * d_l + d_a * d_a + d_b * d_b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_argb() {
        let argb = Argb(0xff00ff00); // Green
        let point = PointProviderLab::from_argb(argb);
        let lab = argb.to_lab();
        assert_eq!(point, [lab.l, lab.a, lab.b]);
    }

    #[test]
    fn test_to_argb() {
        // Red color that is within sRGB gamut and relatively stable in Lab
        let point = [53.23288, 80.1093, 67.22]; // Roughly Argb(0xffff0000)
        let argb = PointProviderLab::to_argb(point);
        let lab = argb.to_lab();
        // Since it's a conversion to Argb (8-bit per channel), there will be some precision loss
        // L* is usually quite stable.
        assert!((lab.l - point[0]).abs() < 1.0, "L difference too large: {}", (lab.l - point[0]).abs());
        assert!((lab.a - point[1]).abs() < 1.0, "a difference too large: {}", (lab.a - point[1]).abs());
        assert!((lab.b - point[2]).abs() < 1.0, "b difference too large: {}", (lab.b - point[2]).abs());
    }

    #[test]
    fn test_distance() {
        let a = [10.0, 20.0, 30.0];
        let b = [12.0, 18.0, 35.0];
        let dist = PointProviderLab::distance(a, b);
        let expected = 2.0 * 2.0 + (-2.0) * (-2.0) + 5.0 * 5.0;
        assert_eq!(dist, expected);
    }

    #[test]
    fn test_back_and_forth() {
        let argb = Argb(0xff00ff00); // Green
        let point = PointProviderLab::from_argb(argb);
        let argb_again = PointProviderLab::to_argb(point);
        assert_eq!(argb_again, argb);
    }
}
