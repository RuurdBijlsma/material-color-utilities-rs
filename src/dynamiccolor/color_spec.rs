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

use std::sync::Arc;

use crate::dynamiccolor::color_spec_2021::ColorSpec2021;
use crate::dynamiccolor::color_spec_2025::ColorSpec2025;
use crate::dynamiccolor::color_spec_2026::ColorSpec2026;
use crate::dynamiccolor::dynamic_color::DynamicColor;
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;

/// All available spec versions, ordered oldest → newest.
///
/// The `PartialOrd` / `Ord` derivations are intentional: `DynamicColor::extend_spec_version`
/// uses `>=` to decide which branch of a color definition applies at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecVersion {
    Spec2021,
    Spec2025,
    Spec2026,
}

/// The device platform that the scheme is being generated for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Phone,
    Watch,
}

/// An interface defining all the necessary methods that could differ between
/// Material Design color-system specification versions.
///
/// Rust equivalent of the Kotlin `ColorSpec` interface.  Every method that
/// returns a `DynamicColor` property is represented as a `fn(&self) ->
/// Arc<DynamicColor>` so that callers can hold onto the colors cheaply.
/// Colors that are optional in the spec (e.g. `primaryDim`) return
/// `Option<Arc<DynamicColor>>`.
pub trait ColorSpec: Send + Sync {
    // ────────────────────────────────────────────────────────────────────────
    // Main palette key colors
    // ────────────────────────────────────────────────────────────────────────
    fn primary_palette_key_color(&self) -> Arc<DynamicColor>;
    fn secondary_palette_key_color(&self) -> Arc<DynamicColor>;
    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor>;
    fn neutral_palette_key_color(&self) -> Arc<DynamicColor>;
    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor>;
    fn error_palette_key_color(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Surfaces [S]
    // ────────────────────────────────────────────────────────────────────────
    fn background(&self) -> Arc<DynamicColor>;
    fn on_background(&self) -> Arc<DynamicColor>;
    fn surface(&self) -> Arc<DynamicColor>;
    fn surface_dim(&self) -> Arc<DynamicColor>;
    fn surface_bright(&self) -> Arc<DynamicColor>;
    fn surface_container_lowest(&self) -> Arc<DynamicColor>;
    fn surface_container_low(&self) -> Arc<DynamicColor>;
    fn surface_container(&self) -> Arc<DynamicColor>;
    fn surface_container_high(&self) -> Arc<DynamicColor>;
    fn surface_container_highest(&self) -> Arc<DynamicColor>;
    fn on_surface(&self) -> Arc<DynamicColor>;
    fn surface_variant(&self) -> Arc<DynamicColor>;
    fn on_surface_variant(&self) -> Arc<DynamicColor>;
    fn inverse_surface(&self) -> Arc<DynamicColor>;
    fn inverse_on_surface(&self) -> Arc<DynamicColor>;
    fn outline(&self) -> Arc<DynamicColor>;
    fn outline_variant(&self) -> Arc<DynamicColor>;
    fn shadow(&self) -> Arc<DynamicColor>;
    fn scrim(&self) -> Arc<DynamicColor>;
    fn surface_tint(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Primaries [P]
    // ────────────────────────────────────────────────────────────────────────
    fn primary(&self) -> Arc<DynamicColor>;
    /// Optional: only present in specs that define a "dim" primary variant.
    fn primary_dim(&self) -> Option<Arc<DynamicColor>>;
    fn on_primary(&self) -> Arc<DynamicColor>;
    fn primary_container(&self) -> Arc<DynamicColor>;
    fn on_primary_container(&self) -> Arc<DynamicColor>;
    fn inverse_primary(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Secondaries [Q]
    // ────────────────────────────────────────────────────────────────────────
    fn secondary(&self) -> Arc<DynamicColor>;
    fn secondary_dim(&self) -> Option<Arc<DynamicColor>>;
    fn on_secondary(&self) -> Arc<DynamicColor>;
    fn secondary_container(&self) -> Arc<DynamicColor>;
    fn on_secondary_container(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Tertiaries [T]
    // ────────────────────────────────────────────────────────────────────────
    fn tertiary(&self) -> Arc<DynamicColor>;
    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>>;
    fn on_tertiary(&self) -> Arc<DynamicColor>;
    fn tertiary_container(&self) -> Arc<DynamicColor>;
    fn on_tertiary_container(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Errors [E]
    // ────────────────────────────────────────────────────────────────────────
    fn error(&self) -> Arc<DynamicColor>;
    fn error_dim(&self) -> Option<Arc<DynamicColor>>;
    fn on_error(&self) -> Arc<DynamicColor>;
    fn error_container(&self) -> Arc<DynamicColor>;
    fn on_error_container(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Primary Fixed Colors [PF]
    // ────────────────────────────────────────────────────────────────────────
    fn primary_fixed(&self) -> Arc<DynamicColor>;
    fn primary_fixed_dim(&self) -> Arc<DynamicColor>;
    fn on_primary_fixed(&self) -> Arc<DynamicColor>;
    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Secondary Fixed Colors [QF]
    // ────────────────────────────────────────────────────────────────────────
    fn secondary_fixed(&self) -> Arc<DynamicColor>;
    fn secondary_fixed_dim(&self) -> Arc<DynamicColor>;
    fn on_secondary_fixed(&self) -> Arc<DynamicColor>;
    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Tertiary Fixed Colors [TF]
    // ────────────────────────────────────────────────────────────────────────
    fn tertiary_fixed(&self) -> Arc<DynamicColor>;
    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor>;
    fn on_tertiary_fixed(&self) -> Arc<DynamicColor>;
    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Other
    // ────────────────────────────────────────────────────────────────────────
    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor>;

    // ────────────────────────────────────────────────────────────────────────
    // Color value calculations
    // ────────────────────────────────────────────────────────────────────────

    /// Computes the resolved HCT value for `color` within `scheme`.
    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct;

    /// Computes the resolved tone (0–100) for `color` within `scheme`.
    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64;

    // ────────────────────────────────────────────────────────────────────────
    // Scheme-level palette builders
    //
    // These are called once during DynamicScheme construction to produce the
    // six core TonalPalettes that the scheme stores.
    // ────────────────────────────────────────────────────────────────────────

    fn get_primary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;

    fn get_secondary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;

    fn get_tertiary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;

    fn get_neutral_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;

    fn get_neutral_variant_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;

    fn get_error_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette;
}

// ────────────────────────────────────────────────────────────────────────────
// ColorSpecs dispatch utility
// ────────────────────────────────────────────────────────────────────────────

/// Factory that returns the correct `ColorSpec` implementation for a given
/// `SpecVersion`.  This mirrors the Kotlin companion-object / factory pattern.
pub struct ColorSpecs;

impl ColorSpecs {
    /// Return a boxed `ColorSpec` for the requested `spec_version`.
    ///
    /// Unrecognised / future variants fall through to the latest available
    /// implementation.
    #[must_use]
    pub fn get(spec_version: SpecVersion) -> Box<dyn ColorSpec> {
        match spec_version {
            SpecVersion::Spec2021 => Box::new(ColorSpec2021::new()),
            SpecVersion::Spec2025 => Box::new(ColorSpec2025::new()),
            SpecVersion::Spec2026 => Box::new(ColorSpec2026::new()),
        }
    }
}
