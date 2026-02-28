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
use crate::utils::color_utils::Argb;
use std::collections::HashMap;
use indexmap::IndexMap;

/// Creates a dictionary with keys of colors, and values of count of the color.
#[derive(Debug, Default)]
pub struct QuantizerMap {
    color_to_count: Option<IndexMap<Argb, u32>>,
}

impl QuantizerMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn color_to_count(&self) -> Option<&IndexMap<Argb, u32>> {
        self.color_to_count.as_ref()
    }
}

impl Quantizer for QuantizerMap {
    fn quantize(&mut self, pixels: &[Argb], _max_colors: usize) -> QuantizerResult {
        let mut pixel_by_count = IndexMap::new();
        for &pixel in pixels {
            *pixel_by_count.entry(pixel).or_insert(0) += 1;
        }
        self.color_to_count = Some(pixel_by_count.clone());
        QuantizerResult::new(pixel_by_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_map() {
        let mut quantizer = QuantizerMap::new();
        let pixels = vec![Argb(0xFF0000FF), Argb(0xFF0000FF), Argb(0xFFFF0000)];
        let result = quantizer.quantize(&pixels, 10);

        assert_eq!(result.color_to_count.get(&Argb(0xFF0000FF)), Some(&2));
        assert_eq!(result.color_to_count.get(&Argb(0xFFFF0000)), Some(&1));
        assert_eq!(result.color_to_count.len(), 2);
    }
}
