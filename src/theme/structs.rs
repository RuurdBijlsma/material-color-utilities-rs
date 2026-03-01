use crate::dynamic::color_spec::{Platform, SpecVersion};
use crate::dynamic::variant::Variant;
use crate::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::utils::color_utils::Argb;

pub struct MaterializedTheme {
    pub source_color_hct: Hct,
    pub source_color: Argb,
    pub variant: Variant,
    pub contrast_level: f64,
    pub platform: Platform,
    pub spec_version: SpecVersion,
    pub schemes: MaterializedSchemeGroup,
}

pub struct MaterializedSchemeGroup {
    light: MaterializedScheme,
    dark: MaterializedScheme,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterializedScheme {
    // Metadata and Parameters
    pub is_dark: bool,
    pub source_color_hct: Hct,
    pub variant: Variant,
    pub contrast_level: f64,
    pub platform: Platform,
    pub spec_version: SpecVersion,

    // Tonal Palettes
    pub primary_palette: TonalPalette,
    pub secondary_palette: TonalPalette,
    pub tertiary_palette: TonalPalette,
    pub neutral_palette: TonalPalette,
    pub neutral_variant_palette: TonalPalette,
    pub error_palette: TonalPalette,

    // Surface and Background Colors
    pub background: Argb,
    pub on_background: Argb,
    pub surface: Argb,
    pub surface_dim: Argb,
    pub surface_bright: Argb,
    pub surface_container_lowest: Argb,
    pub surface_container_low: Argb,
    pub surface_container: Argb,
    pub surface_container_high: Argb,
    pub surface_container_highest: Argb,
    pub on_surface: Argb,
    pub surface_variant: Argb,
    pub on_surface_variant: Argb,
    pub inverse_surface: Argb,
    pub inverse_on_surface: Argb,

    // Utility and Decorative
    pub outline: Argb,
    pub outline_variant: Argb,
    pub shadow: Argb,
    pub scrim: Argb,
    pub surface_tint: Argb,

    // Primary Brand Colors
    pub primary: Argb,
    pub on_primary: Argb,
    pub primary_container: Argb,
    pub on_primary_container: Argb,
    pub inverse_primary: Argb,

    // Secondary Brand Colors
    pub secondary: Argb,
    pub on_secondary: Argb,
    pub secondary_container: Argb,
    pub on_secondary_container: Argb,

    // Tertiary Brand Colors
    pub tertiary: Argb,
    pub on_tertiary: Argb,
    pub tertiary_container: Argb,
    pub on_tertiary_container: Argb,

    // Error Colors
    pub error: Argb,
    pub on_error: Argb,
    pub error_container: Argb,
    pub on_error_container: Argb,

    // Fixed Accent Colors
    pub primary_fixed: Argb,
    pub primary_fixed_dim: Argb,
    pub on_primary_fixed: Argb,
    pub on_primary_fixed_variant: Argb,

    pub secondary_fixed: Argb,
    pub secondary_fixed_dim: Argb,
    pub on_secondary_fixed: Argb,
    pub on_secondary_fixed_variant: Argb,

    pub tertiary_fixed: Argb,
    pub tertiary_fixed_dim: Argb,
    pub on_tertiary_fixed: Argb,
    pub on_tertiary_fixed_variant: Argb,
}
