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

use crate::utils::color_utils::Argb;
use std::collections::HashMap;

/// Represents result of a quantizer run.
#[derive(Debug, Clone, Default)]
pub struct QuantizerResult {
    /// Map with keys of colors in ARGB format, values of how many of the input pixels belong to the color.
    pub color_to_count: HashMap<Argb, u32>,
}

impl QuantizerResult {
    pub fn new(color_to_count: HashMap<Argb, u32>) -> Self {
        Self { color_to_count }
    }
}

/// An interface to allow use of different quantization techniques.
pub trait Quantizer {
    fn quantize(&mut self, pixels: &[Argb], max_colors: usize) -> QuantizerResult;
}
