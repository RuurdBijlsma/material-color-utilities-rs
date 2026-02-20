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
use crate::dynamiccolor::color_spec::SpecVersion;
use crate::dynamiccolor::dynamic_color::DynamicColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Phone,
    Watch,
}

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

    pub fn get_hct(&self, dynamic_color: &DynamicColor) -> Hct {
        dynamic_color.get_hct(self)
    }

    pub fn get_argb(&self, dynamic_color: &DynamicColor) -> u32 {
        dynamic_color.get_argb(self)
    }
}
