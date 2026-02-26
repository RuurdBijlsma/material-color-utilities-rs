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

use crate::hct::hct_color::Hct;
use crate::utils::color_utils::Argb;
use crate::utils::math_utils::MathUtils;
use std::collections::HashMap;

struct ScoredHct {
    hct: Hct,
    score: f64,
}

/// Given a large set of colors, remove colors that are unsuitable for a UI theme, and rank the rest
/// based on suitability.
///
/// Enables use of a high cluster count for image quantization, thus ensuring colors aren't muddied,
/// while curating the high cluster count to a much smaller number of appropriate choices.
pub struct Score;

impl Score {
    const TARGET_CHROMA: f64 = 48.0; // A1 Chroma
    const WEIGHT_PROPORTION: f64 = 0.7;
    const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
    const WEIGHT_CHROMA_BELOW: f64 = 0.1;
    const CUTOFF_CHROMA: f64 = 5.0;
    const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;

    #[must_use]
    pub fn score(colors_to_population: &HashMap<Argb, u32>) -> Vec<Argb> {
        // Fallback color is Google Blue.
        Self::score_with_options(colors_to_population, 4, Argb(0xff4285f4), true)
    }

    #[must_use]
    pub fn score_desired(colors_to_population: &HashMap<Argb, u32>, desired: usize) -> Vec<Argb> {
        Self::score_with_options(colors_to_population, desired, Argb(0xff4285f4), true)
    }

    #[must_use]
    pub fn score_fallback(
        colors_to_population: &HashMap<Argb, u32>,
        desired: usize,
        fallback_color_argb: Argb,
    ) -> Vec<Argb> {
        Self::score_with_options(colors_to_population, desired, fallback_color_argb, true)
    }

    /// Given a map with keys of colors and values of how often the color appears, rank the colors
    /// based on suitability for being used for a UI theme.
    ///
    /// # Arguments
    ///
    /// * `colors_to_population`: map with keys of colors and values of how often the color appears,
    ///   usually from a source image.
    /// * `desired`: max count of colors to be returned in the list.
    /// * `fallback_color_argb`: color to be returned if no other options available.
    /// * `filter`: whether to filter out undesireable combinations.
    ///
    /// # Returns
    ///
    /// Colors sorted by suitability for a UI theme. The most suitable color is the first item,
    /// the least suitable is the last. There will always be at least one color returned. If all the
    /// input colors were not suitable for a theme, a default fallback color will be provided.
    #[must_use]
    pub fn score_with_options(
        colors_to_population: &HashMap<Argb, u32>,
        desired: usize,
        fallback_color_argb: Argb,
        filter: bool,
    ) -> Vec<Argb> {
        // Get the HCT color for each Argb value, while finding the per hue count and
        // total count.
        let mut colors_hct: Vec<Hct> = Vec::with_capacity(colors_to_population.len());
        let mut hue_population = [0u32; 360];
        let mut population_sum = 0.0;

        for (&argb, &population) in colors_to_population {
            let hct = Hct::from_int(argb);
            colors_hct.push(hct);
            let hue = hct.hue().floor() as i32;
            let sanitized_hue = MathUtils::sanitize_degrees_int(hue) as usize;
            hue_population[sanitized_hue] += population;
            population_sum += f64::from(population);
        }

        // Hues with more usage in neighboring 30 degree slice get a larger number.
        let mut hue_excited_proportions = [0.0; 360];
        for hue in 0..360 {
            let proportion = f64::from(hue_population[hue]) / population_sum;
            for i in (hue as i32 - 14)..(hue as i32 + 16) {
                let neighbor_hue = MathUtils::sanitize_degrees_int(i) as usize;
                hue_excited_proportions[neighbor_hue] += proportion;
            }
        }

        // Scores each HCT color based on usage and chroma, while optionally
        // filtering out values that do not have enough chroma or usage.
        let mut scored_hcts: Vec<ScoredHct> = Vec::new();
        for hct in colors_hct {
            let hue = MathUtils::sanitize_degrees_int(hct.hue().round() as i32) as usize;
            let proportion = hue_excited_proportions[hue];

            if filter
                && (hct.chroma() < Self::CUTOFF_CHROMA
                    || proportion <= Self::CUTOFF_EXCITED_PROPORTION)
            {
                continue;
            }

            let proportion_score = proportion * 100.0 * Self::WEIGHT_PROPORTION;
            let chroma_weight = if hct.chroma() < Self::TARGET_CHROMA {
                Self::WEIGHT_CHROMA_BELOW
            } else {
                Self::WEIGHT_CHROMA_ABOVE
            };
            let chroma_score = (hct.chroma() - Self::TARGET_CHROMA) * chroma_weight;
            let score = proportion_score + chroma_score;
            scored_hcts.push(ScoredHct { hct, score });
        }

        // Sorted so that colors with higher scores come first.
        scored_hcts.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Iterates through potential hue differences in degrees in order to select
        // the colors with the largest distribution of hues possible. Starting at
        // 90 degrees(maximum difference for 4 colors) then decreasing down to a
        // 15 degree minimum.
        let mut chosen_colors: Vec<Hct> = Vec::new();
        for difference_degrees in (15..=90).rev() {
            chosen_colors.clear();
            for entry in &scored_hcts {
                let hct = entry.hct;
                let mut has_duplicate_hue = false;
                for chosen_hct in &chosen_colors {
                    if MathUtils::difference_degrees(hct.hue(), chosen_hct.hue())
                        < f64::from(difference_degrees)
                    {
                        has_duplicate_hue = true;
                        break;
                    }
                }
                if !has_duplicate_hue {
                    chosen_colors.push(hct);
                }
                if chosen_colors.len() >= desired {
                    break;
                }
            }
            if chosen_colors.len() >= desired {
                break;
            }
        }

        if chosen_colors.is_empty() {
            return vec![fallback_color_argb];
        }

        chosen_colors.into_iter().map(|h| h.to_int()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::color_utils::Argb;

    #[test]
    fn test_score_default() {
        let colors = HashMap::new();
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors);
        assert_eq!(result, vec![fallback]);
    }

    #[test]
    fn test_score_empty() {
        let colors = HashMap::new();
        let fallback = Argb(0xff4285f4);
        let result = Score::score_with_options(&colors, 4, fallback, true);
        assert_eq!(result, vec![fallback]);
    }

    #[test]
    fn test_score_red() {
        let mut colors = HashMap::new();
        colors.insert(Argb(0xffff0000), 100);
        let fallback = Argb(0xff4285f4);
        let result = Score::score_with_options(&colors, 4, fallback, true);
        // Pure red has enough chroma and proportion
        assert_eq!(result[0], Argb(0xffff0000));
    }

    #[test]
    fn test_score_ranked() {
        let mut colors = HashMap::new();
        colors.insert(Argb(0xFFCCDDCC), 50);
        colors.insert(Argb(0xff00DD88), 50);
        colors.insert(Argb(0xFFCCDDEE), 50);
        let fallback = Argb(0xff4285f4);
        let result = Score::score_with_options(&colors, 4, fallback, true);
        assert_eq!(result[0], Argb(0xff00DD88));
    }

    #[test]
    fn test_score_filtering() {
        let mut colors = HashMap::new();
        // Low chroma color
        colors.insert(Argb(0xff111111), 100);
        let fallback = Argb(0xff4285f4);
        let result = Score::score_with_options(&colors, 4, fallback, true);
        // Should be filtered out, returning fallback
        assert_eq!(result, vec![fallback]);
    }
}
