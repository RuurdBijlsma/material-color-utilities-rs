use crate::utils::color_utils::Argb;
use indexmap::IndexMap;

/// Represents result of a quantizer run.
#[derive(Debug, Clone, Default)]
pub struct QuantizerResult {
    /// Map with keys of colors in ARGB format, values of how many of the input pixels belong to the color.
    pub color_to_count: IndexMap<Argb, u32>,
}

impl QuantizerResult {
    #[must_use]
    pub const fn new(color_to_count: IndexMap<Argb, u32>) -> Self {
        Self { color_to_count }
    }
}

/// An interface to allow use of different quantization techniques.
pub trait Quantizer {
    fn quantize(&mut self, pixels: &[Argb], max_colors: usize) -> QuantizerResult;
}
