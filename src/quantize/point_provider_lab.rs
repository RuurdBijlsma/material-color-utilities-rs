use crate::quantize::point_provider::PointProvider;
use crate::utils::color_utils::{Argb, Lab};

/// Provides conversions needed for K-Means quantization using the L*a*b* color space.
#[derive(Default, Debug, Clone, Copy)]
pub struct PointProviderLab;

impl PointProvider for PointProviderLab {
    fn from_argb(&self, argb: Argb) -> [f64; 3] {
        let lab = argb.to_lab();
        [lab.l, lab.a, lab.b]
    }

    fn to_argb(&self, point: [f64; 3]) -> Argb {
        Argb::from_lab(Lab {
            l: point[0],
            a: point[1],
            b: point[2],
        })
    }

    fn distance(&self, a: [f64; 3], b: [f64; 3]) -> f64 {
        // Zip the coordinates together and sum the squares of their differences.
        // For a fixed-size array of 3, the compiler unrolls this perfectly.
        a.iter()
            .zip(b.iter())
            .map(|(v1, v2)| (v1 - v2).powi(2))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_argb() {
        let provider = PointProviderLab;
        let argb = Argb(0xff00ff00); // Green
        let point = provider.from_argb(argb);
        let lab = argb.to_lab();

        assert_eq!(point, [lab.l, lab.a, lab.b]);
    }

    #[test]
    fn test_to_argb() {
        let provider = PointProviderLab;
        // Red color within sRGB gamut
        let point = [53.23288, 80.1093, 67.22];
        let argb = provider.to_argb(point);
        let lab = argb.to_lab();

        // L*a*b* <-> ARGB conversions involve precision loss (8-bit rounding).
        // We use a small epsilon for float comparisons.
        let epsilon = 1.0;
        assert!((lab.l - point[0]).abs() < epsilon);
        assert!((lab.a - point[1]).abs() < epsilon);
        assert!((lab.b - point[2]).abs() < epsilon);
    }

    #[test]
    fn test_distance() {
        let provider = PointProviderLab;
        let a = [10.0, 20.0, 30.0];
        let b = [12.0, 18.0, 35.0];
        let dist = provider.distance(a, b);

        let expected = 2.0f64.powi(2) + (-2.0f64).powi(2) + 5.0f64.powi(2);
        assert_eq!(dist, expected);
    }

    #[test]
    fn test_back_and_forth() {
        let provider = PointProviderLab;
        let argb = Argb(0xff00ff00);
        let point = provider.from_argb(argb);
        let argb_again = provider.to_argb(point);

        assert_eq!(argb_again, argb);
    }
}
