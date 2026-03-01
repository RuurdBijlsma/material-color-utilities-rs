use crate::hct::hct_color::Hct;

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
    #[must_use]
    pub fn is_disliked(hct: &Hct) -> bool {
        let hue_passes = hct.hue().round() >= 90.0 && hct.hue().round() <= 111.0;
        let chroma_passes = hct.chroma().round() > 16.0;
        let tone_passes = hct.tone().round() < 65.0;
        hue_passes && chroma_passes && tone_passes
    }

    /// If color is disliked, lighten it to make it likable.
    #[must_use]
    pub fn fix_if_disliked(hct: Hct) -> Hct {
        if Self::is_disliked(&hct) {
            Hct::new(hct.hue(), hct.chroma(), 70.0)
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
        let disliked = Hct::new(100.0, 50.0, 50.0);
        assert!(DislikeAnalyzer::is_disliked(&disliked));

        // A liked color: blue
        let liked = Hct::new(250.0, 50.0, 50.0);
        assert!(!DislikeAnalyzer::is_disliked(&liked));

        // Light yellow-green (liked)
        let light = Hct::new(100.0, 50.0, 80.0);
        assert!(!DislikeAnalyzer::is_disliked(&light));
    }

    #[test]
    fn test_fix_if_disliked() {
        let disliked = Hct::new(100.0, 50.0, 50.0);
        let fixed = DislikeAnalyzer::fix_if_disliked(disliked);

        assert!(!DislikeAnalyzer::is_disliked(&fixed));
        assert!((fixed.tone() - 70.0).abs() < 1.0);
    }
}
