use crate::palettes::tonal_palette::TonalPalette;

/// Comprises foundational palettes to build a color scheme.
///
/// Generated from a source color, these palettes will then be part of a [`DynamicScheme`] together
/// with appearance preferences.
pub struct CorePalettes {
    pub primary: TonalPalette,
    pub secondary: TonalPalette,
    pub tertiary: TonalPalette,
    pub neutral: TonalPalette,
    pub neutral_variant: TonalPalette,
}
