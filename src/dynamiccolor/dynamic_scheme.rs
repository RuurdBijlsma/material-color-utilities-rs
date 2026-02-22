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

use crate::dynamiccolor::color_spec::{Platform, SpecVersion};
use crate::dynamiccolor::dynamic_color::DynamicColor;
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::utils::math_utils::MathUtils;
use std::sync::OnceLock;
use crate::dynamiccolor::material_dynamic_colors::MaterialDynamicColors;
use crate::utils::color_utils::Argb;

/// Provides important settings for creating colors dynamically, and 6 color palettes.
#[derive(Debug)]
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
        Self::new_with_platform_and_spec(
            source_color_hct,
            variant,
            is_dark,
            contrast_level,
            Platform::Phone,
            SpecVersion::Spec2021,
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette,
        )
    }

    pub fn new_with_platform_and_spec(
        source_color_hct: Hct,
        variant: Variant,
        is_dark: bool,
        contrast_level: f64,
        platform: Platform,
        spec_version: SpecVersion,
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
            platform,
            spec_version: Self::maybe_fallback_spec_version(spec_version, variant),
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette,
        }
    }

    pub fn from_scheme(other: &DynamicScheme, is_dark: bool) -> Self {
        Self::from_scheme_with_contrast(other, is_dark, other.contrast_level)
    }

    pub fn from_scheme_with_contrast(
        other: &DynamicScheme,
        is_dark: bool,
        contrast_level: f64,
    ) -> Self {
        Self {
            source_color_hct_list: other.source_color_hct_list.clone(),
            variant: other.variant,
            is_dark,
            contrast_level,
            platform: other.platform,
            spec_version: other.spec_version,
            primary_palette: other.primary_palette.clone(),
            secondary_palette: other.secondary_palette.clone(),
            tertiary_palette: other.tertiary_palette.clone(),
            neutral_palette: other.neutral_palette.clone(),
            neutral_variant_palette: other.neutral_variant_palette.clone(),
            error_palette: other.error_palette.clone(),
        }
    }

    /// Returns the primary source color in HCT.
    pub fn source_color_hct(&self) -> &Hct {
        &self.source_color_hct_list[0]
    }

    pub fn source_color_argb(&self) -> Argb {
        self.source_color_hct().to_int()
    }

    pub fn get_hct(&self, dynamic_color: &DynamicColor) -> Hct {
        dynamic_color.get_hct(self)
    }

    pub fn get_argb(&self, dynamic_color: &DynamicColor) -> Argb {
        dynamic_color.get_argb(self)
    }

    pub fn get_piecewise_value(
        source_color_hct: &Hct,
        hue_breakpoints: &[f64],
        hues: &[f64],
    ) -> f64 {
        let size = (hue_breakpoints.len().saturating_sub(1)).min(hues.len());
        let source_hue = source_color_hct.hue();

        for i in 0..size {
            if source_hue >= hue_breakpoints[i] && source_hue < hue_breakpoints[i + 1] {
                return MathUtils::sanitize_degrees_double(hues[i]);
            }
        }

        // No condition matched, return the source value.
        source_hue
    }

    /// Given a hue and set of hue thresholds / rotations, returns the rotated hue.
    pub fn get_rotated_hue(
        source_color_hct: &Hct,
        hue_breakpoints: &[f64],
        rotations: &[f64],
    ) -> f64 {
        let mut rotation = Self::get_piecewise_value(source_color_hct, hue_breakpoints, rotations);
        let size = (hue_breakpoints.len().saturating_sub(1)).min(rotations.len());
        if size == 0 {
            rotation = 0.0;
        }
        MathUtils::sanitize_degrees_double(source_color_hct.hue() + rotation)
    }

    fn maybe_fallback_spec_version(spec_version: SpecVersion, variant: Variant) -> SpecVersion {
        if variant == Variant::Cmf {
            return spec_version;
        }
        if variant == Variant::Expressive
            || variant == Variant::Vibrant
            || variant == Variant::TonalSpot
            || variant == Variant::Neutral
        {
            if spec_version == SpecVersion::Spec2026 {
                return SpecVersion::Spec2025;
            } else {
                return spec_version;
            }
        }
        SpecVersion::Spec2021
    }

    pub fn primary_palette_key_color(&self) -> Argb {
        self.get_argb(&dynamic_colors().primary_palette_key_color())
    }

    pub fn secondary_palette_key_color(&self) -> Argb {
        self.get_argb(&dynamic_colors().secondary_palette_key_color())
    }

    pub fn tertiary_palette_key_color(&self) -> Argb {
        self.get_argb(&dynamic_colors().tertiary_palette_key_color())
    }

    pub fn neutral_palette_key_color(&self) -> Argb {
        self.get_argb(&dynamic_colors().neutral_palette_key_color())
    }

    pub fn neutral_variant_palette_key_color(&self) -> Argb {
        self.get_argb(&dynamic_colors().neutral_variant_palette_key_color())
    }

    pub fn background(&self) -> Argb {
        self.get_argb(&dynamic_colors().background())
    }

    pub fn on_background(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_background())
    }

    pub fn surface(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface())
    }

    pub fn surface_dim(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_dim())
    }

    pub fn surface_bright(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_bright())
    }

    pub fn surface_container_lowest(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_container_lowest())
    }

    pub fn surface_container_low(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_container_low())
    }

    pub fn surface_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_container())
    }

    pub fn surface_container_high(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_container_high())
    }

    pub fn surface_container_highest(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_container_highest())
    }

    pub fn on_surface(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_surface())
    }

    pub fn surface_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_variant())
    }

    pub fn on_surface_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_surface_variant())
    }

    pub fn inverse_surface(&self) -> Argb {
        self.get_argb(&dynamic_colors().inverse_surface())
    }

    pub fn inverse_on_surface(&self) -> Argb {
        self.get_argb(&dynamic_colors().inverse_on_surface())
    }

    pub fn outline(&self) -> Argb {
        self.get_argb(&dynamic_colors().outline())
    }

    pub fn outline_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().outline_variant())
    }

    pub fn shadow(&self) -> Argb {
        self.get_argb(&dynamic_colors().shadow())
    }

    pub fn scrim(&self) -> Argb {
        self.get_argb(&dynamic_colors().scrim())
    }

    pub fn surface_tint(&self) -> Argb {
        self.get_argb(&dynamic_colors().surface_tint())
    }

    pub fn primary(&self) -> Argb {
        self.get_argb(&dynamic_colors().primary())
    }

    pub fn on_primary(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_primary())
    }

    pub fn primary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().primary_container())
    }

    pub fn on_primary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_primary_container())
    }

    pub fn inverse_primary(&self) -> Argb {
        self.get_argb(&dynamic_colors().inverse_primary())
    }

    pub fn secondary(&self) -> Argb {
        self.get_argb(&dynamic_colors().secondary())
    }

    pub fn on_secondary(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_secondary())
    }

    pub fn secondary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().secondary_container())
    }

    pub fn on_secondary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_secondary_container())
    }

    pub fn tertiary(&self) -> Argb {
        self.get_argb(&dynamic_colors().tertiary())
    }

    pub fn on_tertiary(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_tertiary())
    }

    pub fn tertiary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().tertiary_container())
    }

    pub fn on_tertiary_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_tertiary_container())
    }

    pub fn error(&self) -> Argb {
        self.get_argb(&dynamic_colors().error())
    }

    pub fn on_error(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_error())
    }

    pub fn error_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().error_container())
    }

    pub fn on_error_container(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_error_container())
    }

    pub fn primary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().primary_fixed())
    }

    pub fn primary_fixed_dim(&self) -> Argb {
        self.get_argb(&dynamic_colors().primary_fixed_dim())
    }

    pub fn on_primary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_primary_fixed())
    }

    pub fn on_primary_fixed_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_primary_fixed_variant())
    }

    pub fn secondary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().secondary_fixed())
    }

    pub fn secondary_fixed_dim(&self) -> Argb {
        self.get_argb(&dynamic_colors().secondary_fixed_dim())
    }

    pub fn on_secondary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_secondary_fixed())
    }

    pub fn on_secondary_fixed_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_secondary_fixed_variant())
    }

    pub fn tertiary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().tertiary_fixed())
    }

    pub fn tertiary_fixed_dim(&self) -> Argb {
        self.get_argb(&dynamic_colors().tertiary_fixed_dim())
    }

    pub fn on_tertiary_fixed(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_tertiary_fixed())
    }

    pub fn on_tertiary_fixed_variant(&self) -> Argb {
        self.get_argb(&dynamic_colors().on_tertiary_fixed_variant())
    }
}

fn dynamic_colors() -> &'static MaterialDynamicColors {
    static DYNAMIC_COLORS: OnceLock<MaterialDynamicColors> = OnceLock::new();
    DYNAMIC_COLORS.get_or_init(MaterialDynamicColors::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::color_utils::Argb;

    #[test]
    fn test_get_piecewise_value() {
        let hct = Hct::from_int(Argb(0xff0000ff)); // Blue, hue ~265
        let hue_breakpoints = [0.0, 100.0, 200.0, 300.0, 360.0];
        let values = [10.0, 20.0, 30.0, 40.0];

        // Should fall into 200.0..300.0 bucket
        assert_eq!(
            DynamicScheme::get_piecewise_value(&hct, &hue_breakpoints, &values),
            30.0
        );
    }

    #[test]
    fn test_get_rotated_hue() {
        let hct = Hct::from_int(Argb(0xff0000ff)); // Blue, hue ~265.8
        let hue_breakpoints = [0.0, 100.0, 200.0, 300.0, 360.0];
        let rotations = [10.0, 20.0, -30.0, 40.0];

        // Should fall into 200.0..300.0 bucket, rotation is -30
        let expected_hue = MathUtils::sanitize_degrees_double(hct.hue() - 30.0);
        
        let rotated = DynamicScheme::get_rotated_hue(&hct, &hue_breakpoints, &rotations);
        assert!((rotated - expected_hue).abs() < 1e-4);
    }
}


