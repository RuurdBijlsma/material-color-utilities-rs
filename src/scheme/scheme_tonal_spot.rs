use crate::dynamic::color_spec::{Platform, SpecVersion};
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use bon::bon;

/// A calm theme, sedated colors that aren't particularly chromatic.
pub struct SchemeTonalSpot;

#[bon]
impl SchemeTonalSpot {
    #[builder]
    pub fn new(
        #[builder(start_fn, into)] source_color: Hct,
        #[builder(start_fn)] is_dark: bool,
        #[builder(start_fn)] contrast_level: f64,
        #[builder(default, into)] additional_colors: Vec<Hct>,
        #[builder(default = SpecVersion::Spec2021)] spec_version: SpecVersion,
        #[builder(default = Platform::Phone)] platform: Platform,
    ) -> DynamicScheme {
        let spec = ColorSpecs::get(spec_version).call();
        let mut source_color_hct_list = vec![source_color];
        source_color_hct_list.extend(additional_colors);

        let source_color_hct = &source_color_hct_list[0];
        let mut scheme = DynamicScheme::new_with_platform_and_spec(
            *source_color_hct,
            Variant::TonalSpot,
            is_dark,
            contrast_level,
            platform,
            spec_version,
            spec.get_primary_palette(
                Variant::TonalSpot,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_secondary_palette(
                Variant::TonalSpot,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_tertiary_palette(
                Variant::TonalSpot,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_neutral_palette(
                Variant::TonalSpot,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_neutral_variant_palette(
                Variant::TonalSpot,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
            spec.get_error_palette(
                Variant::TonalSpot,
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
