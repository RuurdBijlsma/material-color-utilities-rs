use crate::hct::hct_color::Hct;
use crate::utils::color_utils::Argb;
use crate::utils::math_utils::MathUtils;
use bon::bon;
use indexmap::IndexMap;

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

#[bon]
impl Score {
    const TARGET_CHROMA: f64 = 48.0;
    const WEIGHT_PROPORTION: f64 = 0.7;
    const WEIGHT_CHROMA_ABOVE: f64 = 0.3;
    const WEIGHT_CHROMA_BELOW: f64 = 0.1;
    const CUTOFF_CHROMA: f64 = 5.0;
    const CUTOFF_EXCITED_PROPORTION: f64 = 0.01;

    /// Given a map with keys of colors and values of how often the color appears, rank the colors
    /// based on suitability for being used for a UI theme.
    ///
    /// # Arguments
    ///
    /// * `colors_to_population`: map with keys of colors and values of how often the color appears,
    ///   usually from a source image.
    /// * `desired`: max count of colors to be returned in the list. Defaults to `4`.
    /// * `fallback_color_argb`: color to be returned if no other options available.
    ///   Defaults to Google Blue (`0xff4285f4`).
    /// * `filter`: whether to filter out undesirable combinations. Defaults to `true`.
    ///
    /// # Returns
    ///
    /// Colors sorted by suitability for a UI theme. The most suitable color is the first item,
    /// the least suitable is the last. There will always be at least one color returned. If all the
    /// input colors were not suitable for a theme, a default fallback color will be provided.
    #[builder(start_fn = score)]
    #[must_use]
    pub fn score_impl(
        /// Map with keys of colors and values of how often the color appears (usually from a
        /// source image). This is a required positional argument passed to `Score::score(map)`.
        #[builder(start_fn)]
        colors_to_population: &IndexMap<Argb, u32>,
        /// Max count of colors to be returned. Defaults to `4`.
        #[builder(default = 4)]
        desired_count: usize,
        /// Color to return if no suitable colors are found. Defaults to Google Blue.
        #[builder(default = Argb(0xff4285f4))]
        fallback_color_argb: Argb,
        /// Whether to filter out undesirable combinations. Defaults to `true`.
        #[builder(default = true)]
        filter: bool,
    ) -> Vec<Argb> {
        let mut hue_population = [0u32; 360];
        let mut population_sum = 0.0;

        // 1. Create HCTs and populate hue data
        let colors_hct: Vec<Hct> = colors_to_population
            .iter()
            .map(|(&argb, &population)| {
                let hct = Hct::from_argb(argb);
                let hue = MathUtils::sanitize_degrees_int(hct.hue().floor() as i32) as usize;
                hue_population[hue] += population;
                population_sum += f64::from(population);
                hct
            })
            .collect();

        if population_sum == 0.0 {
            return vec![fallback_color_argb];
        }

        // 2. Calculate excited proportions (Exact neighborhood logic)
        let mut hue_excited_proportions = [0.0; 360];
        for (hue, &pop) in hue_population.iter().enumerate() {
            let proportion = f64::from(pop) / population_sum;
            for i in (hue as i32 - 14)..(hue as i32 + 16) {
                let neighbor_hue = MathUtils::sanitize_degrees_int(i) as usize;
                hue_excited_proportions[neighbor_hue] += proportion;
            }
        }

        // 3. Score and Filter
        let mut scored_hcts: Vec<ScoredHct> = colors_hct
            .into_iter()
            .filter_map(|hct| {
                let hue = MathUtils::sanitize_degrees_int(hct.hue().round() as i32) as usize;
                let proportion = hue_excited_proportions[hue];

                if filter && (hct.chroma() < Self::CUTOFF_CHROMA || proportion <= Self::CUTOFF_EXCITED_PROPORTION) {
                    return None;
                }

                let chroma_score = (hct.chroma() - Self::TARGET_CHROMA) * if hct.chroma() < Self::TARGET_CHROMA {
                    Self::WEIGHT_CHROMA_BELOW
                } else {
                    Self::WEIGHT_CHROMA_ABOVE
                };

                Some(ScoredHct {
                    hct,
                    score: (proportion * 100.0).mul_add(Self::WEIGHT_PROPORTION, chroma_score),
                })
            })
            .collect();

        // Stable sort is required to match original tie-breaking behavior
        scored_hcts.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // 4. Selection Logic (The greedy spread)
        let mut chosen_colors: Vec<Hct> = Vec::with_capacity(desired_count);
        for difference_degrees in (15..=90).rev() {
            chosen_colors.clear();
            for entry in &scored_hcts {
                let has_duplicate = chosen_colors.iter().any(|chosen| {
                    MathUtils::difference_degrees(entry.hct.hue(), chosen.hue()) < f64::from(difference_degrees)
                });

                if !has_duplicate {
                    chosen_colors.push(entry.hct);
                }
                if chosen_colors.len() >= desired_count {
                    break;
                }
            }
            if chosen_colors.len() >= desired_count {
                break;
            }
        }

        if chosen_colors.is_empty() {
            return vec![fallback_color_argb];
        }

        chosen_colors.into_iter().map(|h| h.to_argb()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::color_utils::Argb;

    #[test]
    fn test_score_default() {
        let colors = IndexMap::new();
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors).call();
        assert_eq!(result, vec![fallback]);
    }

    #[test]
    fn test_score_empty() {
        let colors = IndexMap::new();
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors)
            .desired_count(4)
            .fallback_color_argb(fallback)
            .filter(true)
            .call();
        assert_eq!(result, vec![fallback]);
    }

    #[test]
    fn test_score_red() {
        let mut colors = IndexMap::new();
        colors.insert(Argb(0xffff0000), 100);
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors)
            .desired_count(4)
            .fallback_color_argb(fallback)
            .filter(true)
            .call();
        // Pure red has enough chroma and proportion
        assert_eq!(result[0], Argb(0xffff0000));
    }

    #[test]
    fn test_score_ranked() {
        let mut colors = IndexMap::new();
        colors.insert(Argb(0xFFCCDDCC), 50);
        colors.insert(Argb(0xff00DD88), 50);
        colors.insert(Argb(0xFFCCDDEE), 50);
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors)
            .desired_count(4)
            .fallback_color_argb(fallback)
            .filter(true)
            .call();
        assert_eq!(result[0], Argb(0xff00DD88));
    }

    #[test]
    fn test_score_filtering() {
        let mut colors = IndexMap::new();
        // Low chroma color
        colors.insert(Argb(0xff111111), 100);
        let fallback = Argb(0xff4285f4);
        let result = Score::score(&colors)
            .desired_count(4)
            .fallback_color_argb(fallback)
            .filter(true)
            .call();
        // Should be filtered out, returning fallback
        assert_eq!(result, vec![fallback]);
    }
}
