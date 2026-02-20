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

use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::dynamiccolor::variant::Variant;
use crate::dynamiccolor::color_spec::{Platform, SpecVersion};
use crate::dynamiccolor::dynamic_color::DynamicColor;

/// Provides important settings for creating colors dynamically, and 6 color palettes.
pub struct DynamicScheme {
    pub source_color_hct_list: Vec<Hct>,
    pub variant: Variant,
    pub is_dark: bool,
    pub contrast_level: f64,
    pub platform: Platform,
    pub spec_version: SpecVersion,
    pub primary_palette: TonalPalette,
    pub secondary_palette: TonalPalette,
    pub tertiary_palette: TonalPalette,
    pub neutral_palette: TonalPalette,
    pub neutral_variant_palette: TonalPalette,
    pub error_palette: TonalPalette,
}

impl DynamicScheme {
    pub fn new(
        source_color_hct: Hct,
        variant: Variant,
        is_dark: bool,
        contrast_level: f64,
        primary_palette: TonalPalette,
        secondary_palette: TonalPalette,
        tertiary_palette: TonalPalette,
        neutral_palette: TonalPalette,
        neutral_variant_palette: TonalPalette,
        error_palette: TonalPalette,
    ) -> Self {
        Self {
            source_color_hct_list: vec![source_color_hct],
            variant,
            is_dark,
            contrast_level,
            platform: Platform::Phone,
            spec_version: SpecVersion::Spec2021,
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette,
        }
    }

    /// Returns the primary source color in HCT.
    pub fn source_color_hct(&self) -> &Hct {
        &self.source_color_hct_list[0]
    }

    pub fn get_hct(&self, dynamic_color: &DynamicColor) -> Hct {
        dynamic_color.get_hct(self)
    }

    pub fn get_argb(&self, dynamic_color: &DynamicColor) -> u32 {
        dynamic_color.get_argb(self)
    }

    /// Given a hue and set of hue thresholds / rotations, returns the rotated hue.
    ///
    /// This mirrors the Kotlin `DynamicScheme.getRotatedHue` companion function used by
    /// color specs to compute hue-rotated tertiary / secondary palette hues for some
    /// variants (Expressive, Vibrant, …).
    ///
    /// `source_color_hct`: the source HCT used for the scheme.
    /// `hues`: sorted array of hue thresholds (must contain one more entry than `rotations`).
    /// `rotations`: amount to rotate the hue for each segment.
    pub fn get_rotated_hue(source_color_hct: &Hct, hues: &[f64], rotations: &[f64]) -> f64 {
        let source_hue = source_color_hct.hue();
        if rotations.len() + 1 != hues.len() {
            // Malformed input — just return the source hue
            return source_hue;
        }
        for i in 0..rotations.len() {
            let this_hue = hues[i];
            let next_hue = hues[i + 1];
            if this_hue <= source_hue && source_hue < next_hue {
                return crate::utils::math_utils::MathUtils::sanitize_degrees_double(
                    source_hue + rotations[i],
                );
            }
        }
        source_hue
    }
}
