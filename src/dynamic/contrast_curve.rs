use crate::utils::math_utils::MathUtils;

/// A class containing a value that changes with the contrast level.
///
/// Usually represents the contrast requirements for a dynamic color on its background. The four
/// values correspond to values for contrast levels -1.0, 0.0, 0.5, and 1.0, respectively.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastCurve {
    /// Value for contrast level -1.0
    pub low: f64,
    /// Value for contrast level 0.0
    pub normal: f64,
    /// Value for contrast level 0.5
    pub medium: f64,
    /// Value for contrast level 1.0
    pub high: f64,
}

impl ContrastCurve {
    #[must_use]
    pub const fn new(low: f64, normal: f64, medium: f64, high: f64) -> Self {
        Self {
            low,
            normal,
            medium,
            high,
        }
    }

    /// Returns the value at a given contrast level.
    ///
    /// # Arguments
    /// * `contrast_level` - The contrast level. 0.0 is the default (normal); -1.0 is the lowest; 1.0
    ///   is the highest.
    ///
    /// # Returns
    /// The value. For contrast ratios, a number between 1.0 and 21.0.
    #[must_use]
    pub fn get(&self, contrast_level: f64) -> f64 {
        if contrast_level <= -1.0 {
            self.low
        } else if contrast_level < 0.0 {
            MathUtils::lerp(self.low, self.normal, contrast_level + 1.0)
        } else if contrast_level < 0.5 {
            MathUtils::lerp(self.normal, self.medium, contrast_level / 0.5)
        } else if contrast_level < 1.0 {
            MathUtils::lerp(self.medium, self.high, (contrast_level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_curve_get() {
        let curve = ContrastCurve::new(3.0, 4.5, 7.0, 11.0);

        // Exact levels
        assert_eq!(curve.get(-1.0), 3.0);
        assert_eq!(curve.get(0.0), 4.5);
        assert_eq!(curve.get(0.5), 7.0);
        assert_eq!(curve.get(1.0), 11.0);

        // Out of bounds
        assert_eq!(curve.get(-2.0), 3.0);
        assert_eq!(curve.get(2.0), 11.0);

        // Interpolation
        assert!((curve.get(-0.5) - 3.75).abs() < 1e-9);
        assert!((curve.get(0.25) - 5.75).abs() < 1e-9);
        assert!((curve.get(0.75) - 9.0).abs() < 1e-9);
    }
}
