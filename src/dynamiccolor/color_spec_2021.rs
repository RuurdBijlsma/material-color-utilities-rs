use crate::dynamiccolor::color_spec::ColorSpec;
use crate::dynamiccolor::dynamic_color::DynamicColor;
use crate::hct::Hct;
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;

/// Minimal implementation of ColorSpec2021 for foundation.
pub struct ColorSpec2021;

impl ColorSpec2021 {
    pub fn new() -> Self {
        Self
    }
}

impl ColorSpec for ColorSpec2021 {
    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        // Minimal implementation: use the initial tone from the palette
        let tone = (color.tone)(scheme);
        let palette = (color.palette)(scheme);
        palette.get_hct(tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        self.get_hct(scheme, color).tone()
    }
}
