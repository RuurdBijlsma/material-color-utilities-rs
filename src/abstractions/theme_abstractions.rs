use crate::abstractions::{MaterializedScheme, MaterializedSchemeGroup, MaterializedTheme};
use crate::dynamic::color_spec::{Platform, SpecVersion};
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::material_dynamic_colors::MaterialDynamicColors;
use crate::dynamic::variant::Variant;
use crate::scheme::{
    SchemeCmf, SchemeContent, SchemeExpressive, SchemeFidelity, SchemeFruitSalad, SchemeMonochrome,
    SchemeNeutral, SchemeRainbow, SchemeTonalSpot, SchemeVibrant,
};
use crate::utils::color_utils::Argb;

/// Generates a materialized theme from a source color.
#[bon::builder]
pub fn get_theme_from_color(
    /// The source color.
    #[builder(start_fn)]
    source_color: Argb,
    /// Which variant to use
    #[builder(default = Variant::Vibrant)]
    variant: Variant,
    /// Contrast level.
    ///  - `0.0` for default contrast.
    ///  - `0.5` for medium contrast.
    ///  - `1.0` for highest contrast.
    ///  - `-1.0` for reduced contrast.
    #[builder(default = 0.0)]
    contrast_level: f64,
    /// `SpecVersion` tracks which version of the Material Design dynamic color spec the algorithms are following.
    #[builder(default = SpecVersion::Spec2026)]
    spec_version: SpecVersion,
    /// What platform to optimize colors for.
    #[builder(default = Platform::Phone)]
    platform: Platform,
) -> MaterializedTheme {
    let light_scheme = create_dynamic_scheme(
        source_color,
        variant,
        false,
        contrast_level,
        spec_version,
        platform,
    );
    let dark_scheme = create_dynamic_scheme(
        source_color,
        variant,
        true,
        contrast_level,
        spec_version,
        platform,
    );

    let mdc = MaterialDynamicColors::new_with_spec(spec_version);

    MaterializedTheme {
        source_color,
        variant,
        contrast_level,
        platform,
        spec_version,
        schemes: MaterializedSchemeGroup {
            light: materialize(&light_scheme, &mdc),
            dark: materialize(&dark_scheme, &mdc),
        },
    }
}

/// Helper to map the Variant enum to the specific Scheme builder.
fn create_dynamic_scheme(
    source_color: Argb,
    variant: Variant,
    is_dark: bool,
    contrast_level: f64,
    spec_version: SpecVersion,
    platform: Platform,
) -> DynamicScheme {
    match variant {
        Variant::TonalSpot => SchemeTonalSpot::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Vibrant => SchemeVibrant::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Expressive => SchemeExpressive::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Content => SchemeContent::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Fidelity => SchemeFidelity::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Monochrome => SchemeMonochrome::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Neutral => SchemeNeutral::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Rainbow => SchemeRainbow::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::FruitSalad => SchemeFruitSalad::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
        Variant::Cmf => SchemeCmf::builder(source_color, is_dark, contrast_level)
            .spec_version(spec_version)
            .platform(platform)
            .build(),
    }
}

/// Extracts all ARGB values from a `DynamicScheme` into a `MaterializedScheme`.
fn materialize(scheme: &DynamicScheme, mdc: &MaterialDynamicColors) -> MaterializedScheme {
    MaterializedScheme {
        is_dark: scheme.is_dark,
        source_color: scheme.source_color_argb(),
        variant: scheme.variant,
        contrast_level: scheme.contrast_level,
        platform: scheme.platform,
        spec_version: scheme.spec_version,

        primary_palette: scheme.primary_palette.clone(),
        secondary_palette: scheme.secondary_palette.clone(),
        tertiary_palette: scheme.tertiary_palette.clone(),
        neutral_palette: scheme.neutral_palette.clone(),
        neutral_variant_palette: scheme.neutral_variant_palette.clone(),
        error_palette: scheme.error_palette.clone(),

        background: scheme.get_argb(&mdc.background()),
        on_background: scheme.get_argb(&mdc.on_background()),
        surface: scheme.get_argb(&mdc.surface()),
        surface_dim: scheme.get_argb(&mdc.surface_dim()),
        surface_bright: scheme.get_argb(&mdc.surface_bright()),
        surface_container_lowest: scheme.get_argb(&mdc.surface_container_lowest()),
        surface_container_low: scheme.get_argb(&mdc.surface_container_low()),
        surface_container: scheme.get_argb(&mdc.surface_container()),
        surface_container_high: scheme.get_argb(&mdc.surface_container_high()),
        surface_container_highest: scheme.get_argb(&mdc.surface_container_highest()),
        on_surface: scheme.get_argb(&mdc.on_surface()),
        surface_variant: scheme.get_argb(&mdc.surface_variant()),
        on_surface_variant: scheme.get_argb(&mdc.on_surface_variant()),
        inverse_surface: scheme.get_argb(&mdc.inverse_surface()),
        inverse_on_surface: scheme.get_argb(&mdc.inverse_on_surface()),

        outline: scheme.get_argb(&mdc.outline()),
        outline_variant: scheme.get_argb(&mdc.outline_variant()),
        shadow: scheme.get_argb(&mdc.shadow()),
        scrim: scheme.get_argb(&mdc.scrim()),
        surface_tint: scheme.get_argb(&mdc.surface_tint()),

        primary: scheme.get_argb(&mdc.primary()),
        on_primary: scheme.get_argb(&mdc.on_primary()),
        primary_container: scheme.get_argb(&mdc.primary_container()),
        on_primary_container: scheme.get_argb(&mdc.on_primary_container()),
        inverse_primary: scheme.get_argb(&mdc.inverse_primary()),

        secondary: scheme.get_argb(&mdc.secondary()),
        on_secondary: scheme.get_argb(&mdc.on_secondary()),
        secondary_container: scheme.get_argb(&mdc.secondary_container()),
        on_secondary_container: scheme.get_argb(&mdc.on_secondary_container()),

        tertiary: scheme.get_argb(&mdc.tertiary()),
        on_tertiary: scheme.get_argb(&mdc.on_tertiary()),
        tertiary_container: scheme.get_argb(&mdc.tertiary_container()),
        on_tertiary_container: scheme.get_argb(&mdc.on_tertiary_container()),

        error: scheme.get_argb(&mdc.error()),
        on_error: scheme.get_argb(&mdc.on_error()),
        error_container: scheme.get_argb(&mdc.error_container()),
        on_error_container: scheme.get_argb(&mdc.on_error_container()),

        primary_fixed: scheme.get_argb(&mdc.primary_fixed()),
        primary_fixed_dim: scheme.get_argb(&mdc.primary_fixed_dim()),
        on_primary_fixed: scheme.get_argb(&mdc.on_primary_fixed()),
        on_primary_fixed_variant: scheme.get_argb(&mdc.on_primary_fixed_variant()),

        secondary_fixed: scheme.get_argb(&mdc.secondary_fixed()),
        secondary_fixed_dim: scheme.get_argb(&mdc.secondary_fixed_dim()),
        on_secondary_fixed: scheme.get_argb(&mdc.on_secondary_fixed()),
        on_secondary_fixed_variant: scheme.get_argb(&mdc.on_secondary_fixed_variant()),

        tertiary_fixed: scheme.get_argb(&mdc.tertiary_fixed()),
        tertiary_fixed_dim: scheme.get_argb(&mdc.tertiary_fixed_dim()),
        on_tertiary_fixed: scheme.get_argb(&mdc.on_tertiary_fixed()),
        on_tertiary_fixed_variant: scheme.get_argb(&mdc.on_tertiary_fixed_variant()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::variant::Variant;
    use crate::hct::Hct;
    use crate::utils::color_utils::Argb;

    const GOOGLE_BLUE: Argb = Argb(0xFF4285F4);

    #[test]
    fn test_theme_generation_basic() {
        // Test with default values
        let theme = get_theme_from_color(GOOGLE_BLUE).call();

        assert_eq!(theme.source_color, GOOGLE_BLUE);
        assert_eq!(theme.variant, Variant::Vibrant); // Default

        // Ensure light and dark schemes were generated
        assert!(!theme.schemes.light.is_dark);
        assert!(theme.schemes.dark.is_dark);

        // Ensure key colors are not transparent/zero
        assert_ne!(theme.schemes.light.primary.0, 0);
        assert_ne!(theme.schemes.dark.primary.0, 0);
    }

    #[test]
    fn test_theme_variant_monochrome() {
        let theme = get_theme_from_color(GOOGLE_BLUE)
            .variant(Variant::Monochrome)
            .call();

        assert_eq!(theme.variant, Variant::Monochrome);

        // In Monochrome, chroma should be 0 or very close to 0
        let primary_hct = Hct::from_argb(theme.schemes.light.primary);
        assert!(primary_hct.chroma() < 2.0);

        let secondary_hct = Hct::from_argb(theme.schemes.light.secondary);
        assert!(secondary_hct.chroma() < 2.0);
    }

    #[test]
    fn test_contrast_levels_affect_output() {
        let low_contrast = get_theme_from_color(GOOGLE_BLUE)
            .contrast_level(-1.0)
            .call();

        let high_contrast = get_theme_from_color(GOOGLE_BLUE).contrast_level(1.0).call();

        // High contrast primary should be different from low contrast primary
        assert_ne!(
            low_contrast.schemes.light.primary,
            high_contrast.schemes.light.primary
        );

        // In light mode, high contrast "on" colors are typically darker (lower L*)
        let on_primary_low = Hct::from_argb(low_contrast.schemes.light.on_primary).tone();
        let on_primary_high = Hct::from_argb(high_contrast.schemes.light.on_primary).tone();

        assert!(on_primary_high < on_primary_low);
    }

    #[test]
    fn test_light_dark_contrast_polarization() {
        let theme = get_theme_from_color(GOOGLE_BLUE).call();

        let light_bg = Hct::from_argb(theme.schemes.light.background).tone();
        let dark_bg = Hct::from_argb(theme.schemes.dark.background).tone();

        // Background in light mode must be significantly lighter than dark mode
        assert!(light_bg > 80.0);
        assert!(dark_bg < 20.0);
    }

    #[test]
    fn test_fixed_colors_presence() {
        let theme = get_theme_from_color(GOOGLE_BLUE).call();
        let light = &theme.schemes.light;

        // Fixed colors should be generated regardless of light/dark mode
        // and usually follow specific Material 3 tonal guidelines
        assert_ne!(light.primary_fixed.0, 0);
        assert_ne!(light.secondary_fixed_dim.0, 0);

        let tone_fixed = Hct::from_argb(light.primary_fixed).tone();
        assert!(tone_fixed > 40. && tone_fixed < 70.);
    }

    #[test]
    fn test_spec_version_consistency() {
        let theme_2021 = get_theme_from_color(GOOGLE_BLUE)
            .spec_version(SpecVersion::Spec2021)
            .call();

        let theme_2026 = get_theme_from_color(GOOGLE_BLUE)
            .spec_version(SpecVersion::Spec2026)
            .call();

        // The algorithms for surfaces changed between 2021 and 2026
        assert_ne!(
            theme_2021.schemes.light.surface_container,
            theme_2026.schemes.light.surface_container
        );
    }
}
