use crate::dynamiccolor::color_spec::{Platform, SpecVersion};
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;

/// A Dynamic Color theme with 2 source colors.
pub struct SchemeCmf;

impl SchemeCmf {
    pub fn new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme {
        Self::new_with_platform_and_spec(
            source_color_hct,
            is_dark,
            contrast_level,
            SpecVersion::Spec2026,
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
        if spec_version != SpecVersion::Spec2026 {
            panic!("SchemeCmf can only be used with spec version 2026.");
        }

        let source_color_hct = source_color_hct_list[0];

        let primary_palette =
            TonalPalette::from_hue_and_chroma(source_color_hct.hue(), source_color_hct.chroma());
        let secondary_palette = TonalPalette::from_hue_and_chroma(
            source_color_hct.hue(),
            source_color_hct.chroma() * 0.5,
        );
        let tertiary_palette = Self::tertiary_palette(&source_color_hct_list);
        let neutral_palette = TonalPalette::from_hue_and_chroma(
            source_color_hct.hue(),
            source_color_hct.chroma() * 0.2,
        );
        let neutral_variant_palette = TonalPalette::from_hue_and_chroma(
            source_color_hct.hue(),
            source_color_hct.chroma() * 0.2,
        );
        let error_palette =
            TonalPalette::from_hue_and_chroma(23.0, source_color_hct.chroma().max(50.0));

        let mut scheme = DynamicScheme::new_with_platform_and_spec(
            source_color_hct,
            Variant::Cmf,
            is_dark,
            contrast_level,
            platform,
            spec_version,
            primary_palette,
            secondary_palette,
            tertiary_palette,
            neutral_palette,
            neutral_variant_palette,
            error_palette,
        );

        scheme.source_color_hct_list = source_color_hct_list;
        scheme
    }

    fn tertiary_palette(source_color_hct_list: &[Hct]) -> TonalPalette {
        let source_color_hct = &source_color_hct_list[0];
        let secondary_source_color_hct = source_color_hct_list.get(1).unwrap_or(source_color_hct);

        if source_color_hct.to_int() == secondary_source_color_hct.to_int() {
            TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                source_color_hct.chroma() * 0.75,
            )
        } else {
            TonalPalette::from_hue_and_chroma(
                secondary_source_color_hct.hue(),
                secondary_source_color_hct.chroma(),
            )
        }
    }
}
