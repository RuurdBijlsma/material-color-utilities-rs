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

use crate::dynamic::color_spec::{ColorSpec, SpecVersion};
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;

/// Named colors, otherwise known as tokens, or roles, in the Material Design system.
pub struct MaterialDynamicColors {
    color_spec: Box<dyn ColorSpec>,
}

impl Default for MaterialDynamicColors {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialDynamicColors {
    /// Constructs a new `MaterialDynamicColors` using the default 2026 color spec.
    #[must_use]
    pub fn new() -> Self {
        Self {
            color_spec: ColorSpecs::get(SpecVersion::Spec2026),
        }
    }

    /// Constructs a new `MaterialDynamicColors` using the specified color spec version.
    #[must_use]
    pub fn new_with_spec(spec_version: SpecVersion) -> Self {
        Self {
            color_spec: ColorSpecs::get(spec_version),
        }
    }

    #[must_use]
    pub fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        self.color_spec.highest_surface(scheme)
    }

    // ////////////////////////////////////////////////////////////////
    // Main Palettes //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn primary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.primary_palette_key_color()
    }

    #[must_use]
    pub fn secondary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.secondary_palette_key_color()
    }

    #[must_use]
    pub fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.tertiary_palette_key_color()
    }

    #[must_use]
    pub fn neutral_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.neutral_palette_key_color()
    }

    #[must_use]
    pub fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.neutral_variant_palette_key_color()
    }

    #[must_use]
    pub fn error_palette_key_color(&self) -> Arc<DynamicColor> {
        self.color_spec.error_palette_key_color()
    }

    // ////////////////////////////////////////////////////////////////
    // Surfaces [S] //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn background(&self) -> Arc<DynamicColor> {
        self.color_spec.background()
    }

    #[must_use]
    pub fn on_background(&self) -> Arc<DynamicColor> {
        self.color_spec.on_background()
    }

    #[must_use]
    pub fn surface(&self) -> Arc<DynamicColor> {
        self.color_spec.surface()
    }

    #[must_use]
    pub fn surface_dim(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_dim()
    }

    #[must_use]
    pub fn surface_bright(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_bright()
    }

    #[must_use]
    pub fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_container_lowest()
    }

    #[must_use]
    pub fn surface_container_low(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_container_low()
    }

    #[must_use]
    pub fn surface_container(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_container()
    }

    #[must_use]
    pub fn surface_container_high(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_container_high()
    }

    #[must_use]
    pub fn surface_container_highest(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_container_highest()
    }

    #[must_use]
    pub fn on_surface(&self) -> Arc<DynamicColor> {
        self.color_spec.on_surface()
    }

    #[must_use]
    pub fn surface_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_variant()
    }

    #[must_use]
    pub fn on_surface_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.on_surface_variant()
    }

    #[must_use]
    pub fn inverse_surface(&self) -> Arc<DynamicColor> {
        self.color_spec.inverse_surface()
    }

    #[must_use]
    pub fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        self.color_spec.inverse_on_surface()
    }

    #[must_use]
    pub fn outline(&self) -> Arc<DynamicColor> {
        self.color_spec.outline()
    }

    #[must_use]
    pub fn outline_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.outline_variant()
    }

    #[must_use]
    pub fn shadow(&self) -> Arc<DynamicColor> {
        self.color_spec.shadow()
    }

    #[must_use]
    pub fn scrim(&self) -> Arc<DynamicColor> {
        self.color_spec.scrim()
    }

    #[must_use]
    pub fn surface_tint(&self) -> Arc<DynamicColor> {
        self.color_spec.surface_tint()
    }

    // ////////////////////////////////////////////////////////////////
    // Primaries [P] //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn primary(&self) -> Arc<DynamicColor> {
        self.color_spec.primary()
    }

    #[must_use]
    pub fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.color_spec.primary_dim()
    }

    #[must_use]
    pub fn on_primary(&self) -> Arc<DynamicColor> {
        self.color_spec.on_primary()
    }

    #[must_use]
    pub fn primary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.primary_container()
    }

    #[must_use]
    pub fn on_primary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.on_primary_container()
    }

    #[must_use]
    pub fn inverse_primary(&self) -> Arc<DynamicColor> {
        self.color_spec.inverse_primary()
    }

    // ///////////////////////////////////////////////////////////////
    // Primary Fixed Colors [PF] //
    // ///////////////////////////////////////////////////////////////
    #[must_use]
    pub fn primary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.primary_fixed()
    }

    #[must_use]
    pub fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        self.color_spec.primary_fixed_dim()
    }

    #[must_use]
    pub fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.on_primary_fixed()
    }

    #[must_use]
    pub fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.on_primary_fixed_variant()
    }

    // ////////////////////////////////////////////////////////////////
    // Secondaries [Q] //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn secondary(&self) -> Arc<DynamicColor> {
        self.color_spec.secondary()
    }

    #[must_use]
    pub fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.color_spec.secondary_dim()
    }

    #[must_use]
    pub fn on_secondary(&self) -> Arc<DynamicColor> {
        self.color_spec.on_secondary()
    }

    #[must_use]
    pub fn secondary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.secondary_container()
    }

    #[must_use]
    pub fn on_secondary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.on_secondary_container()
    }

    // ///////////////////////////////////////////////////////////////
    // Secondary Fixed Colors [QF] //
    // ///////////////////////////////////////////////////////////////
    #[must_use]
    pub fn secondary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.secondary_fixed()
    }

    #[must_use]
    pub fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        self.color_spec.secondary_fixed_dim()
    }

    #[must_use]
    pub fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.on_secondary_fixed()
    }

    #[must_use]
    pub fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.on_secondary_fixed_variant()
    }

    // ////////////////////////////////////////////////////////////////
    // Tertiaries [T] //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn tertiary(&self) -> Arc<DynamicColor> {
        self.color_spec.tertiary()
    }

    #[must_use]
    pub fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.color_spec.tertiary_dim()
    }

    #[must_use]
    pub fn on_tertiary(&self) -> Arc<DynamicColor> {
        self.color_spec.on_tertiary()
    }

    #[must_use]
    pub fn tertiary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.tertiary_container()
    }

    #[must_use]
    pub fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        self.color_spec.on_tertiary_container()
    }

    // ///////////////////////////////////////////////////////////////
    // Tertiary Fixed Colors [TF] //
    // ///////////////////////////////////////////////////////////////
    #[must_use]
    pub fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.tertiary_fixed()
    }

    #[must_use]
    pub fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        self.color_spec.tertiary_fixed_dim()
    }

    #[must_use]
    pub fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        self.color_spec.on_tertiary_fixed()
    }

    #[must_use]
    pub fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        self.color_spec.on_tertiary_fixed_variant()
    }

    // ////////////////////////////////////////////////////////////////
    // Errors [E] //
    // ////////////////////////////////////////////////////////////////
    #[must_use]
    pub fn error(&self) -> Arc<DynamicColor> {
        self.color_spec.error()
    }

    #[must_use]
    pub fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        self.color_spec.error_dim()
    }

    #[must_use]
    pub fn on_error(&self) -> Arc<DynamicColor> {
        self.color_spec.on_error()
    }

    #[must_use]
    pub fn error_container(&self) -> Arc<DynamicColor> {
        self.color_spec.error_container()
    }

    #[must_use]
    pub fn on_error_container(&self) -> Arc<DynamicColor> {
        self.color_spec.on_error_container()
    }

    // ////////////////////////////////////////////////////////////////
    // All Colors //
    // ////////////////////////////////////////////////////////////////
    /// All dynamic colors in Material Design system.
    pub fn all_dynamic_colors(&self) -> Vec<Box<dyn Fn() -> Option<Arc<DynamicColor>> + '_>> {
        COLOR_GETTERS
            .iter()
            .map(|&getter| {
                let closure: Box<dyn Fn() -> Option<Arc<DynamicColor>> + '_> =
                    Box::new(move || getter(self));
                closure
            })
            .collect()
    }
}

pub type ColorGetter = fn(&MaterialDynamicColors) -> Option<Arc<DynamicColor>>;

pub const COLOR_GETTERS: &[ColorGetter] = &[
    |m| Some(m.primary_palette_key_color()),
    |m| Some(m.secondary_palette_key_color()),
    |m| Some(m.tertiary_palette_key_color()),
    |m| Some(m.neutral_palette_key_color()),
    |m| Some(m.neutral_variant_palette_key_color()),
    |m| Some(m.error_palette_key_color()),
    |m| Some(m.background()),
    |m| Some(m.on_background()),
    |m| Some(m.surface()),
    |m| Some(m.surface_dim()),
    |m| Some(m.surface_bright()),
    |m| Some(m.surface_container_lowest()),
    |m| Some(m.surface_container_low()),
    |m| Some(m.surface_container()),
    |m| Some(m.surface_container_high()),
    |m| Some(m.surface_container_highest()),
    |m| Some(m.on_surface()),
    |m| Some(m.surface_variant()),
    |m| Some(m.on_surface_variant()),
    |m| Some(m.outline()),
    |m| Some(m.outline_variant()),
    |m| Some(m.inverse_surface()),
    |m| Some(m.inverse_on_surface()),
    |m| Some(m.shadow()),
    |m| Some(m.scrim()),
    |m| Some(m.surface_tint()),
    |m| Some(m.primary()),
    |m| m.primary_dim(),
    |m| Some(m.on_primary()),
    |m| Some(m.primary_container()),
    |m| Some(m.on_primary_container()),
    |m| Some(m.primary_fixed()),
    |m| Some(m.primary_fixed_dim()),
    |m| Some(m.on_primary_fixed()),
    |m| Some(m.on_primary_fixed_variant()),
    |m| Some(m.inverse_primary()),
    |m| Some(m.secondary()),
    |m| m.secondary_dim(),
    |m| Some(m.on_secondary()),
    |m| Some(m.secondary_container()),
    |m| Some(m.on_secondary_container()),
    |m| Some(m.secondary_fixed()),
    |m| Some(m.secondary_fixed_dim()),
    |m| Some(m.on_secondary_fixed()),
    |m| Some(m.on_secondary_fixed_variant()),
    |m| Some(m.tertiary()),
    |m| m.tertiary_dim(),
    |m| Some(m.on_tertiary()),
    |m| Some(m.tertiary_container()),
    |m| Some(m.on_tertiary_container()),
    |m| Some(m.tertiary_fixed()),
    |m| Some(m.tertiary_fixed_dim()),
    |m| Some(m.on_tertiary_fixed()),
    |m| Some(m.on_tertiary_fixed_variant()),
    |m| Some(m.error()),
    |m| m.error_dim(),
    |m| Some(m.on_error()),
    |m| Some(m.error_container()),
    |m| Some(m.on_error_container()),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_dynamic_colors() {
        use crate::dynamic::color_spec::SpecVersion;
        use crate::dynamic::color_specs::ColorSpecs;

        let mdc = MaterialDynamicColors {
            color_spec: ColorSpecs::get(SpecVersion::Spec2021),
        };

        // Ensure all colors resolve to correct count, including options that might be None.
        let colors = mdc.all_dynamic_colors();
        assert_eq!(colors.len(), 59);

        // Basic spot-check
        assert!(colors[0]().is_some());
        assert!(colors[10]().is_some());
    }
}
