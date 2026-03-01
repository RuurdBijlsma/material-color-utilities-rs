use crate::dynamic::color_spec::{Platform, SpecVersion};
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use bon::bon;

/// A Dynamic Color theme with 2 source colors.
pub struct SchemeCmf;

#[bon]
impl SchemeCmf {
    #[builder]
    pub fn new(
        #[builder(start_fn, into)] source_color: Hct,
        #[builder(start_fn)] is_dark: bool,
        #[builder(start_fn)] contrast_level: f64,
        #[builder(default, into)] additional_colors: Vec<Hct>,
        #[builder(default = SpecVersion::Spec2026)] spec_version: SpecVersion,
        #[builder(default = Platform::Phone)] platform: Platform,
    ) -> DynamicScheme {
        let mut source_color_hct_list = vec![source_color];
        source_color_hct_list.extend(additional_colors);

        let source_color_hct = &source_color_hct_list[0];

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
            *source_color_hct,
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

        if source_color_hct.to_argb() == secondary_source_color_hct.to_argb() {
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
