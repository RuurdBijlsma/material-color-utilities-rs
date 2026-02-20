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

use crate::quantize::quantizer::{Quantizer, QuantizerResult};
use crate::quantize::quantizer_wsmeans::QuantizerWsmeans;
use crate::quantize::quantizer_wu::QuantizerWu;
use crate::utils::color_utils::Argb;

/// An image quantizer that improves on the quality of a standard K-Means algorithm by setting the
/// K-Means initial state to the output of a Wu quantizer, instead of random centroids. Improves on
/// speed by several optimizations, as implemented in Wsmeans, or Weighted Square Means, K-Means with
/// those optimizations.
///
/// This algorithm was designed by M. Emre Celebi, and was found in their 2011 paper, Improving the
/// Performance of K-Means for Color Quantization. https://arxiv.org/abs/1101.0395
#[derive(Default)]
pub struct QuantizerCelebi;

impl QuantizerCelebi {
    pub fn new() -> Self {
        Self
    }
}

impl Quantizer for QuantizerCelebi {
    /// Reduce the number of colors needed to represented the input, minimizing the difference between
    /// the original image and the recolored image.
    ///
    /// # Arguments
    /// * `pixels` - Colors in ARGB format.
    /// * `max_colors` - The number of colors to divide the image into. A lower number of colors may be
    ///   returned.
    ///
    /// # Returns
    /// `QuantizerResult` with keys of colors in ARGB format, and values of number of pixels in the original
    /// image that correspond to the color in the quantized image.
    fn quantize(&mut self, pixels: &[Argb], max_colors: usize) -> QuantizerResult {
        let mut wu = QuantizerWu::new();
        let wu_result = wu.quantize(pixels, max_colors);

        let starting_clusters: Vec<Argb> = wu_result.color_to_count.keys().cloned().collect();

        let clusters = QuantizerWsmeans::quantize(pixels, &starting_clusters, max_colors);
        QuantizerResult::new(clusters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hct::Hct;

    #[test]
    fn test_quantize_celebi_basic() {
        let pixels = vec![
            Argb(0xFFFF0000), // Red -> Count: 4
            Argb(0xFFFF0000),
            Argb(0xFFFF0000),
            Argb(0xFFDD0000), // orange-red -> quantized to reddish
            Argb(0xFF00FF00), // Green -> Count: 2
            Argb(0xFF00FF00),
            Argb(0xFF0000FF), // Blue -> Count: 3
            Argb(0xFF0000FF),
            Argb(0xFF0000FF),
        ];

        let mut celebi = QuantizerCelebi::new();
        let result = celebi.quantize(&pixels, 3);

        assert!(!result.color_to_count.is_empty());
        assert!(result.color_to_count.len() <= 3);

        // Check if there's 3 blue, 2 green, 4 red
        for (argb, count) in result.color_to_count {
            let hue = Hct::from_int(argb).hue();
            // RED:
            if (0.0 - hue).abs() < 55.0 {
                assert_eq!(count, 4);
                continue
            }
            // GREEN:
            if (120.0 - hue).abs() < 55.0 {
                assert_eq!(count, 2);
                continue
            }
            // BLUE:
            if (240.0 - hue).abs() < 55.0 {
                assert_eq!(count, 3);
                continue
            }
            panic!("Unknown color in result.color_to_count");
        }
    }

    #[test]
    fn test_quantize_celebi_single_color() {
        let pixels = vec![Argb(0xFFFF0000); 10];
        let mut celebi = QuantizerCelebi::new();
        let result = celebi.quantize(&pixels, 128);

        assert_eq!(result.color_to_count.len(), 1);
        assert!(result.color_to_count.contains_key(&Argb(0xFFFF0000)));
    }
}
