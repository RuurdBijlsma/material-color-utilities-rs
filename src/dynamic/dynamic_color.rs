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
use std::fmt;
use std::fmt::Debug;
use crate::contrast::contrast_utils::Contrast;
use crate::dynamic::color_spec::SpecVersion;
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::ToneDeltaPair;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::utils::color_utils::Argb;
use std::sync::Arc;

pub type DynamicColorFunction<T> = Arc<dyn Fn(&DynamicScheme) -> T + Send + Sync>;

/// A color that adjusts itself based on UI state, represented by `DynamicScheme`.
pub struct DynamicColor {
    pub name: String,
    pub palette: DynamicColorFunction<TonalPalette>,
    pub is_background: bool,
    pub chroma_multiplier: Option<DynamicColorFunction<f64>>,
    pub background: Option<DynamicColorFunction<Option<Arc<Self>>>>,
    pub tone: DynamicColorFunction<f64>,
    pub second_background: Option<DynamicColorFunction<Option<Arc<Self>>>>,
    pub contrast_curve: Option<DynamicColorFunction<Option<ContrastCurve>>>,
    pub tone_delta_pair: Option<DynamicColorFunction<Option<ToneDeltaPair>>>,
    pub opacity: Option<DynamicColorFunction<Option<f64>>>,
}

impl Debug for DynamicColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicColor")
            .field("name", &self.name)
            .field("is_background", &self.is_background)
            .field("palette", &"<function>")
            .field("chroma_multiplier", &self.chroma_multiplier.as_ref().map(|_| "<function>"))
            .field("background", &self.background.as_ref().map(|_| "<function>"))
            .field("tone", &"<function>")
            .field("second_background", &self.second_background.as_ref().map(|_| "<function>"))
            .field("contrast_curve", &self.contrast_curve.as_ref().map(|_| "<function>"))
            .field("tone_delta_pair", &self.tone_delta_pair.as_ref().map(|_| "<function>"))
            .field("opacity", &self.opacity.as_ref().map(|_| "<function>"))
            .finish()
    }
}

impl DynamicColor {
    pub fn new(
        name: String,
        palette: DynamicColorFunction<TonalPalette>,
        is_background: bool,
        chroma_multiplier: Option<DynamicColorFunction<f64>>,
        background: Option<DynamicColorFunction<Option<Arc<Self>>>>,
        tone: Option<DynamicColorFunction<f64>>,
        second_background: Option<DynamicColorFunction<Option<Arc<Self>>>>,
        contrast_curve: Option<DynamicColorFunction<Option<ContrastCurve>>>,
        tone_delta_pair: Option<DynamicColorFunction<Option<ToneDeltaPair>>>,
        opacity: Option<DynamicColorFunction<Option<f64>>>,
    ) -> Self {
        // Validation logic from Kotlin init block
        assert!(
            !(background.is_none() && second_background.is_some()),
            "Color {name} has second_background defined, but background is not defined."
        );
        assert!(
            !(background.is_none() && contrast_curve.is_some()),
            "Color {name} has contrast_curve defined, but background is not defined."
        );
        assert!(
            !(background.is_some() && contrast_curve.is_none()),
            "Color {name} has background defined, but contrast_curve is not defined."
        );

        let tone = tone.unwrap_or_else(|| {
            let bg_func = background.clone();
            Arc::new(move |scheme| {
                if let Some(ref bg) = bg_func
                    && let Some(bg_color) = bg(scheme)
                {
                    return bg_color.get_tone(scheme);
                }
                50.0
            })
        });

        Self {
            name,
            palette,
            is_background,
            chroma_multiplier,
            background,
            tone,
            second_background,
            contrast_curve,
            tone_delta_pair,
            opacity,
        }
    }

    pub fn get_argb(&self, scheme: &DynamicScheme) -> Argb {
        let argb = self.get_hct(scheme).to_int();
        dbg!(&argb);
        if let Some(ref opacity_func) = self.opacity
            && let Some(opacity_percentage) = opacity_func(scheme)
        {
            let alpha = (opacity_percentage * 255.0).round() as u32;
            let alpha = alpha.clamp(0, 255);
            return Argb((argb.0 & 0x00ffffff) | (alpha << 24));
        }
        argb
    }

    pub fn get_hct(&self, scheme: &DynamicScheme) -> Hct {
        // TODO: cache here same as DynamicColor.kt
        dbg!(&scheme.spec_version);
        let get_hct_result = ColorSpecs::get(scheme.spec_version).get_hct(scheme, self);
        dbg!(&get_hct_result);
        get_hct_result
    }

    pub fn get_tone(&self, scheme: &DynamicScheme) -> f64 {
        ColorSpecs::get(scheme.spec_version).get_tone(scheme, self)
    }

    /// Create a `DynamicColor` from an ARGB hex code.
    #[must_use]
    pub fn from_argb(name: &str, argb: Argb) -> Self {
        let hct = Hct::from_int(argb);
        let palette = TonalPalette::from_int(argb);
        Self::new(
            name.to_string(),
            Arc::new(move |_| palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(move |_| hct.tone())),
            None,
            None,
            None,
            None::<DynamicColorFunction<Option<f64>>>,
        )
    }

    /// Given a background tone, find a foreground tone, while ensuring they reach a contrast ratio
    /// that is as close to ratio as possible.
    #[must_use]
    pub fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
        let lighter_tone = Contrast::lighter_unsafe(bg_tone, ratio);
        let darker_tone = Contrast::darker_unsafe(bg_tone, ratio);
        let lighter_ratio = Contrast::ratio_of_tones(lighter_tone, bg_tone);
        let darker_ratio = Contrast::ratio_of_tones(darker_tone, bg_tone);
        let prefer_lighter = Self::tone_prefers_light_foreground(bg_tone);

        if prefer_lighter {
            let negligible_difference = (lighter_ratio - darker_ratio).abs() < 0.1
                && lighter_ratio < ratio
                && darker_ratio < ratio;
            if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible_difference {
                lighter_tone
            } else {
                darker_tone
            }
        } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
            darker_tone
        } else {
            lighter_tone
        }
    }

    /// Adjust a tone down such that white has 4.5 contrast, if the tone is reasonably close to
    /// supporting it.
    #[must_use]
    pub fn enable_light_foreground(tone: f64) -> f64 {
        if Self::tone_prefers_light_foreground(tone) && !Self::tone_allows_light_foreground(tone) {
            49.0
        } else {
            tone
        }
    }

    #[must_use]
    pub fn extend_spec_version(
        &self,
        spec_version: SpecVersion,
        extended_color: &Self,
    ) -> Arc<Self> {
        Self::validate_extended_color(self, spec_version, extended_color);

        let this_palette = self.palette.clone();
        let ext_palette = extended_color.palette.clone();
        let palette = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                (ext_palette)(scheme)
            } else {
                (this_palette)(scheme)
            }
        });

        let this_tone = self.tone.clone();
        let ext_tone = extended_color.tone.clone();
        let tone = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                (ext_tone)(scheme)
            } else {
                (this_tone)(scheme)
            }
        });

        let this_chroma = self.chroma_multiplier.clone();
        let ext_chroma = extended_color.chroma_multiplier.clone();
        let chroma_multiplier = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_chroma.as_ref().map_or(1.0, |f| f(scheme))
            } else {
                this_chroma.as_ref().map_or(1.0, |f| f(scheme))
            }
        });

        let this_bg = self.background.clone();
        let ext_bg = extended_color.background.clone();
        let background = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_bg.as_ref().and_then(|f| f(scheme))
            } else {
                this_bg.as_ref().and_then(|f| f(scheme))
            }
        });

        let this_bg2 = self.second_background.clone();
        let ext_bg2 = extended_color.second_background.clone();
        let second_background = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_bg2.as_ref().and_then(|f| f(scheme))
            } else {
                this_bg2.as_ref().and_then(|f| f(scheme))
            }
        });

        let this_curve = self.contrast_curve.clone();
        let ext_curve = extended_color.contrast_curve.clone();
        let contrast_curve = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_curve.as_ref().and_then(|f| f(scheme))
            } else {
                this_curve.as_ref().and_then(|f| f(scheme))
            }
        });

        let this_delta = self.tone_delta_pair.clone();
        let ext_delta = extended_color.tone_delta_pair.clone();
        let tone_delta_pair = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_delta.as_ref().and_then(|f| f(scheme))
            } else {
                this_delta.as_ref().and_then(|f| f(scheme))
            }
        });

        let this_opacity = self.opacity.clone();
        let ext_opacity = extended_color.opacity.clone();
        let opacity = Arc::new(move |scheme: &DynamicScheme| {
            if scheme.spec_version >= spec_version {
                ext_opacity.as_ref().and_then(|f| f(scheme))
            } else {
                this_opacity.as_ref().and_then(|f| f(scheme))
            }
        });

        Arc::new(Self::new(
            self.name.clone(),
            palette,
            self.is_background,
            Some(chroma_multiplier),
            Some(background),
            Some(tone),
            Some(second_background),
            Some(contrast_curve),
            Some(tone_delta_pair),
            Some(opacity),
        ))
    }

    fn validate_extended_color(&self, spec_version: SpecVersion, extended_color: &Self) {
        assert!(
            self.name == extended_color.name,
            "Attempting to extend color {} with color {} of different name for spec version {:?}.",
            self.name,
            extended_color.name,
            spec_version
        );
        assert!(
            self.is_background == extended_color.is_background,
            "Attempting to extend color {} as a {} with color {} as a {} for spec version {:?}.",
            self.name,
            if self.is_background {
                "background"
            } else {
                "foreground"
            },
            extended_color.name,
            if extended_color.is_background {
                "background"
            } else {
                "foreground"
            },
            spec_version
        );
    }

    /// People prefer white foregrounds on ~T60-70.
    #[must_use]
    pub fn tone_prefers_light_foreground(tone: f64) -> bool {
        tone.round() < 60.0
    }

    /// Tones less than ~T50 always permit white at 4.5 contrast.
    #[must_use]
    pub fn tone_allows_light_foreground(tone: f64) -> bool {
        tone.round() <= 49.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_argb() {
        let color = DynamicColor::from_argb("test", Argb(0xff00ff00));
        assert_eq!(color.name, "test");
        // HCT for 0xff00ff00 (pure green) is roughly hue 142, chroma 107, tone 88
        let _hct = Hct::from_int(Argb(0xff00ff00));
        // We can't easily test the closures without a scheme, but we can check initial tone logic
    }

    #[test]
    fn test_foreground_tone() {
        // T90 background, 4.5 ratio -> T42.5 (roughly)
        let fg = DynamicColor::foreground_tone(90.0, 4.5);
        assert!(fg < 45.0 && fg > 40.0);

        // T10 background, 4.5 ratio -> T54.6 (roughly)
        let fg = DynamicColor::foreground_tone(10.0, 4.5);
        assert!(fg > 50.0 && fg < 60.0);
    }

    #[test]
    fn test_tone_preferences() {
        assert!(DynamicColor::tone_prefers_light_foreground(59.0));
        assert!(!DynamicColor::tone_prefers_light_foreground(61.0));
        assert!(DynamicColor::tone_allows_light_foreground(49.0));
        assert!(!DynamicColor::tone_allows_light_foreground(50.0));
    }
}
