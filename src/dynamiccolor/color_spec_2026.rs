use std::sync::Arc;

use crate::dynamiccolor::color_spec::{ColorSpec, Platform};
use crate::dynamiccolor::dynamic_color::DynamicColor;
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;

// ─── ColorSpec2026 ──────────────────────────────────────────────────────────

/// [ColorSpec] implementation for the 2021 Material Design color specification.
pub struct ColorSpec2026;

impl ColorSpec2026 {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorSpec2026 {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpec for ColorSpec2026 {
    fn primary_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn neutral_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn error_palette_key_color(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn background(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn outline(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn shadow(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn scrim(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn surface_tint(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn primary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        todo!()
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        todo!()
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        todo!()
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn error(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        todo!()
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        todo!()
    }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        todo!()
    }

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        todo!()
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        todo!()
    }

    fn get_primary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }

    fn get_secondary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }

    fn get_tertiary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }

    fn get_neutral_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }

    fn get_neutral_variant_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }

    fn get_error_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        todo!()
    }
}