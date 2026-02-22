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

use crate::hct::hct::Hct;

/// Check and/or fix universally disliked colors.
///
/// Color science studies of color preference indicate universal distaste for dark yellow-greens, and
/// also show this is correlated to distaste for biological waste and rotting food.
///
/// See Palmer and Schloss, 2010 or Schloss and Palmer's Chapter 21 in Handbook of Color Psychology
/// (2015).
pub struct DislikeAnalyzer;

impl DislikeAnalyzer {
    /// Returns true if color is disliked.
    ///
    /// Disliked is defined as a dark yellow-green that is not neutral.
    pub fn is_disliked(hct: &Hct) -> bool {
        let hue_passes = hct.hue().round() >= 90.0 && hct.hue().round() <= 111.0;
        let chroma_passes = hct.chroma().round() > 16.0;
        let tone_passes = hct.tone().round() < 65.0;
        hue_passes && chroma_passes && tone_passes
    }

    /// If color is disliked, lighten it to make it likable.
    pub fn fix_if_disliked(hct: Hct) -> Hct {
        if Self::is_disliked(&hct) {
            Hct::from(hct.hue(), hct.chroma(), 70.0)
        } else {
            hct
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_disliked() {
        // A known disliked color: dark yellow-green
        // #897700
        let disliked = Hct::from(100.0, 50.0, 50.0);
        assert!(DislikeAnalyzer::is_disliked(&disliked));

        // A liked color: blue
        let liked = Hct::from(250.0, 50.0, 50.0);
        assert!(!DislikeAnalyzer::is_disliked(&liked));

        // Light yellow-green (liked)
        let light = Hct::from(100.0, 50.0, 80.0);
        assert!(!DislikeAnalyzer::is_disliked(&light));
    }

    #[test]
    fn test_fix_if_disliked() {
        let disliked = Hct::from(100.0, 50.0, 50.0);
        let fixed = DislikeAnalyzer::fix_if_disliked(disliked);

        assert!(!DislikeAnalyzer::is_disliked(&fixed));
        assert!((fixed.tone() - 70.0).abs() < 1.0);
    }
}
