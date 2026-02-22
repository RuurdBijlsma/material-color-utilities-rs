use crate::utils::color_utils::Argb;
/// An interface to allow use of different color spaces by quantizers.
///
/// This trait defines how to map sRGB colors into a coordinate system
/// where distance measurements (Delta E) can be performed.
pub trait PointProvider {
    /// Converts an ARGB color into coordinates in the provider's color space.
    fn from_argb(&self, argb: Argb) -> [f64; 3];

    /// Converts coordinates back into an ARGB color.
    fn to_argb(&self, point: [f64; 3]) -> Argb;

    /// Returns the squared Euclidean distance between two points.
    ///
    /// Squared distance is used as an optimization, as it preserves the
    /// relative ordering required for K-Means without the cost of `sqrt`.
    fn distance(&self, a: [f64; 3], b: [f64; 3]) -> f64;
}
