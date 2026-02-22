use crate::dynamiccolor::color_spec::{Platform, SpecVersion};
use crate::dynamiccolor::color_specs::ColorSpecs;
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;

/// A monochrome theme, colors are purely black / white / gray.
pub struct SchemeMonochrome;

impl SchemeMonochrome {
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        Self::new_with_platform_and_spec(
            source_color_hct,
            is_dark,
            contrast_level,
            SpecVersion::Spec2021,
            Platform::Phone,
        )
    }

    pub fn new_with_platform_and_spec(
        source_color_hct: Hct,
        is_dark: bool,
        contrast_level: f64,
        spec_version: SpecVersion,
        platform: Platform,
    ) -> DynamicScheme {
        Self::new_with_list_and_platform_and_spec(
            vec![source_color_hct],
            is_dark,
            contrast_level,
            spec_version,
            platform,
        )
    }

    pub fn new_with_list_and_platform_and_spec(
        source_color_hct_list: Vec<Hct>,
        is_dark: bool,
        contrast_level: f64,
        spec_version: SpecVersion,
        platform: Platform,
    ) -> DynamicScheme {
        let spec = ColorSpecs::get(spec_version);
        let source_color_hct = &source_color_hct_list[0];
        let mut scheme = DynamicScheme::new_with_platform_and_spec(
            *source_color_hct,
            Variant::Monochrome,
            is_dark,
            contrast_level,
            platform,
            spec_version,
            spec.get_primary_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_secondary_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_tertiary_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_neutral_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_neutral_variant_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_error_palette(
                Variant::Monochrome,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
        );
        scheme.source_color_hct_list = source_color_hct_list;
        scheme
    }
}
