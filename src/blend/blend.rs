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
use crate::hct::hct::Hct;
use crate::utils::color_utils::Argb;
use crate::utils::math_utils::MathUtils;

/// Functions for blending in HCT and CAM16.
pub struct Blend;

impl Blend {
    /// Blend the design color's HCT hue towards the key color's HCT hue, in a way that leaves the
    /// original color recognizable and recognizably shifted towards the key color.
    ///
    /// # Arguments
    ///
    /// * `design_color`: ARGB representation of an arbitrary color.
    /// * `source_color`: ARGB representation of the main theme color.
    ///
    /// # Returns
    ///
    /// The design color with a hue shifted towards the system's color, a slightly
    /// warmer/cooler variant of the design color's hue.
    pub fn harmonize(design_color: Argb, source_color: Argb) -> Argb {
        let from_hct = Hct::from_int(design_color);
        let to_hct = Hct::from_int(source_color);
        let difference_degrees = MathUtils::difference_degrees(from_hct.hue(), to_hct.hue());
        let rotation_degrees = (difference_degrees * 0.5).min(15.0);
        let output_hue = MathUtils::sanitize_degrees_double(
            from_hct.hue()
                + rotation_degrees * MathUtils::rotation_direction(from_hct.hue(), to_hct.hue()),
        );
        Hct::from(output_hue, from_hct.chroma(), from_hct.tone()).to_int()
    }

    /// Blends hue from one color into another. The chroma and tone of the original color are
    /// maintained.
    ///
    /// # Arguments
    ///
    /// * `from`: ARGB representation of color
    /// * `to`: ARGB representation of color
    /// * `amount`: how much blending to perform; 0.0 >= and <= 1.0
    ///
    /// # Returns
    ///
    /// from, with a hue blended towards to. Chroma and tone are constant.
    pub fn hct_hue(from: Argb, to: Argb, amount: f64) -> Argb {
        let ucs = Self::cam16_ucs(from, to, amount);
        let ucs_cam = Cam16::from_int(ucs);
        let from_cam = Cam16::from_int(from);
        let blended = Hct::from(ucs_cam.hue, from_cam.chroma, from.lstar());
        blended.to_int()
    }

    /// Blend in CAM16-UCS space.
    ///
    /// # Arguments
    ///
    /// * `from`: ARGB representation of color
    /// * `to`: ARGB representation of color
    /// * `amount`: how much blending to perform; 0.0 >= and <= 1.0
    ///
    /// # Returns
    ///
    /// from, blended towards to. Hue, chroma, and tone will change.
    pub fn cam16_ucs(from: Argb, to: Argb, amount: f64) -> Argb {
        let from_cam = Cam16::from_int(from);
        let to_cam = Cam16::from_int(to);
        let jstar = MathUtils::lerp(from_cam.jstar, to_cam.jstar, amount);
        let astar = MathUtils::lerp(from_cam.astar, to_cam.astar, amount);
        let bstar = MathUtils::lerp(from_cam.bstar, to_cam.bstar, amount);
        Cam16::from_ucs(jstar, astar, bstar).to_int()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harmonize() {
        let design_color = Argb(0xFFFF0000); // Red
        let source_color = Argb(0xFF0000FF); // Blue
        let harmonized = Blend::harmonize(design_color, source_color);

        let from_hct = Hct::from_int(design_color);
        let result_hct = Hct::from_int(harmonized);

        // Red hue is 27.4, Blue hue is 282.7 in HCT (approx)
        // Rotation should be 15 degrees towards blue.
        assert!((result_hct.hue() - from_hct.hue()).abs() > 0.0);
        // Chroma and tone should be preserved as much as possible,
        // but chroma might be clipped if the new hue doesn't support it.
        assert!((result_hct.tone() - from_hct.tone()).abs() < 1.0);
    }

    #[test]
    fn test_hct_hue() {
        let from = Argb(0xFFFF0000); // Red
        let to = Argb(0xFF00FF00); // Green
        let blended = Blend::hct_hue(from, to, 0.5);

        let from_hct = Hct::from_int(from);
        let result_hct = Hct::from_int(blended);

        assert!((result_hct.hue() - from_hct.hue()).abs() > 0.0);
        // Chroma and tone should be preserved as much as possible,
        // but chroma might be clipped if the new hue doesn't support it.
        assert!((result_hct.tone() - from_hct.tone()).abs() < 1.0);
    }

    #[test]
    fn test_cam16_ucs() {
        let from = Argb(0xFFFF0000); // Red
        let to = Argb(0xFF00FF00); // Green
        let blended = Blend::cam16_ucs(from, to, 0.5);

        let from_cam = Cam16::from_int(from);
        let to_cam = Cam16::from_int(to);
        let result_cam = Cam16::from_int(blended);

        // UCS blending should result in something between the two
        assert!(result_cam.jstar > from_cam.jstar.min(to_cam.jstar));
        assert!(result_cam.jstar < from_cam.jstar.max(to_cam.jstar));
    }
}
