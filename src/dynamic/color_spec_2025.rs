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

use std::sync::Arc;

use crate::contrast::contrast_utils::Contrast;
use crate::dynamic::color_spec::{ColorSpec, Platform, SpecVersion};
use crate::dynamic::color_spec_2021::ColorSpec2021;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::{DynamicColor, DynamicColorFunction};
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

/// [`ColorSpec`] implementation for the 2025 Material Design color specification.
pub struct ColorSpec2025 {
    base: ColorSpec2021,
}

impl Default for ColorSpec2025 {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpec2025 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: ColorSpec2021::new(),
        }
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Helper Methods
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn t_max_c(palette: &TonalPalette, lower_bound: f64, upper_bound: f64, chroma_multiplier: f64) -> f64 {
        let answer = Self::find_best_tone_for_chroma(palette.hue, palette.chroma * chroma_multiplier, 100.0, true);
        answer.clamp(lower_bound, upper_bound)
    }

    fn t_max_c_default(palette: &TonalPalette) -> f64 {
        Self::t_max_c(palette, 0.0, 100.0, 1.0)
    }

    fn t_max_c_bounds(palette: &TonalPalette, lower_bound: f64, upper_bound: f64) -> f64 {
        Self::t_max_c(palette, lower_bound, upper_bound, 1.0)
    }

    fn t_min_c(palette: &TonalPalette, lower_bound: f64, upper_bound: f64) -> f64 {
        let answer = Self::find_best_tone_for_chroma(palette.hue, palette.chroma, 0.0, false);
        answer.clamp(lower_bound, upper_bound)
    }

    fn t_min_c_default(palette: &TonalPalette) -> f64 {
        Self::t_min_c(palette, 0.0, 100.0)
    }

    fn find_best_tone_for_chroma(hue: f64, chroma: f64, mut tone: f64, by_decreasing_tone: bool) -> f64 {
        let mut answer = tone;
        let mut best_candidate = Hct::from(hue, chroma, answer);
        while best_candidate.chroma() < chroma {
            if !(0.0..=100.0).contains(&tone) {
                break;
            }
            tone += if by_decreasing_tone { -1.0 } else { 1.0 };
            let new_candidate = Hct::from(hue, chroma, tone);
            if best_candidate.chroma() < new_candidate.chroma() {
                best_candidate = new_candidate;
                answer = tone;
            }
        }
        answer
    }

    fn get_contrast_curve(default_contrast: f64) -> ContrastCurve {
        if (default_contrast - 1.5).abs() < f64::EPSILON {
            ContrastCurve::new(1.5, 1.5, 3.0, 5.5)
        } else if (default_contrast - 3.0).abs() < f64::EPSILON {
            ContrastCurve::new(3.0, 3.0, 4.5, 7.0)
        } else if (default_contrast - 4.5).abs() < f64::EPSILON {
            ContrastCurve::new(4.5, 4.5, 7.0, 11.0)
        } else if (default_contrast - 6.0).abs() < f64::EPSILON {
            ContrastCurve::new(6.0, 6.0, 7.0, 11.0)
        } else if (default_contrast - 7.0).abs() < f64::EPSILON {
            ContrastCurve::new(7.0, 7.0, 11.0, 21.0)
        } else if (default_contrast - 9.0).abs() < f64::EPSILON {
            ContrastCurve::new(9.0, 9.0, 11.0, 21.0)
        } else if (default_contrast - 11.0).abs() < f64::EPSILON {
            ContrastCurve::new(11.0, 11.0, 21.0, 21.0)
        } else if (default_contrast - 21.0).abs() < f64::EPSILON {
            ContrastCurve::new(21.0, 21.0, 21.0, 21.0)
        } else {
            ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0)
        }
    }

    fn get_expressive_neutral_hue(source_color_hct: &Hct) -> f64 {
        DynamicScheme::get_rotated_hue(
            source_color_hct,
            &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
            &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
        )
    }

    fn get_expressive_neutral_chroma(source_color_hct: &Hct, is_dark: bool, platform: Platform) -> f64 {
        let neutral_hue = Self::get_expressive_neutral_hue(source_color_hct);
        if platform == Platform::Phone {
            if is_dark {
                if Hct::is_yellow(neutral_hue) { 6.0 } else { 14.0 }
            } else {
                18.0
            }
        } else {
            12.0
        }
    }

    fn get_vibrant_neutral_hue(source_color_hct: &Hct) -> f64 {
        DynamicScheme::get_rotated_hue(
            source_color_hct,
            &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
            &[-14.0, 10.0, -14.0, 10.0, -14.0],
        )
    }

    fn get_vibrant_neutral_chroma(source_color_hct: &Hct, platform: Platform) -> f64 {
        let neutral_hue = Self::get_vibrant_neutral_hue(source_color_hct);
        if platform == Platform::Phone {
            28.0
        } else if Hct::is_blue(neutral_hue) {
            28.0
        } else {
            20.0
        }
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Static Tone Evaluators (For breaking cycles in ToneDeltaPairs)
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn primary_tone(scheme: &DynamicScheme) -> f64 {
        match scheme.variant {
            Variant::Neutral => {
                if scheme.platform == Platform::Phone { if scheme.is_dark { 80.0 } else { 40.0 } } else { 90.0 }
            }
            Variant::TonalSpot => {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark { 80.0 } else { Self::t_max_c_default(&scheme.primary_palette) }
                } else {
                    Self::t_max_c_bounds(&scheme.primary_palette, 0.0, 90.0)
                }
            }
            Variant::Expressive => {
                if scheme.platform == Platform::Phone {
                    let hue = scheme.primary_palette.hue;
                    let upper = if Hct::is_yellow(hue) { 25.0 } else if Hct::is_cyan(hue) { 88.0 } else { 98.0 };
                    Self::t_max_c_bounds(&scheme.primary_palette, 0.0, upper)
                } else {
                    Self::t_max_c_default(&scheme.primary_palette)
                }
            }
            _ => { // Vibrant
                if scheme.platform == Platform::Phone {
                    let upper = if Hct::is_cyan(scheme.primary_palette.hue) { 88.0 } else { 98.0 };
                    Self::t_max_c_bounds(&scheme.primary_palette, 0.0, upper)
                } else {
                    Self::t_max_c_default(&scheme.primary_palette)
                }
            }
        }
    }

    fn primary_dim_tone(scheme: &DynamicScheme) -> f64 {
        match scheme.variant {
            Variant::Neutral => 85.0,
            Variant::TonalSpot => Self::t_max_c_bounds(&scheme.primary_palette, 0.0, 90.0),
            _ => Self::t_max_c_default(&scheme.primary_palette),
        }
    }

    fn primary_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            return 30.0;
        }
        match scheme.variant {
            Variant::Neutral => if scheme.is_dark { 30.0 } else { 90.0 },
            Variant::TonalSpot => {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.primary_palette, 35.0, 93.0)
                } else {
                    Self::t_max_c_bounds(&scheme.primary_palette, 0.0, 90.0)
                }
            }
            Variant::Expressive => {
                if scheme.is_dark {
                    Self::t_max_c_bounds(&scheme.primary_palette, 30.0, 93.0)
                } else {
                    let upper = if Hct::is_cyan(scheme.primary_palette.hue) { 88.0 } else { 90.0 };
                    Self::t_max_c_bounds(&scheme.primary_palette, 78.0, upper)
                }
            }
            _ => { // Vibrant
                if scheme.is_dark {
                    Self::t_min_c(&scheme.primary_palette, 66.0, 93.0)
                } else {
                    let upper = if Hct::is_cyan(scheme.primary_palette.hue) { 88.0 } else { 93.0 };
                    Self::t_max_c_bounds(&scheme.primary_palette, 66.0, upper)
                }
            }
        }
    }

    fn secondary_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            if scheme.variant == Variant::Neutral {
                return 90.0;
            }
            return Self::t_max_c_bounds(&scheme.secondary_palette, 0.0, 90.0);
        }
        match scheme.variant {
            Variant::Neutral => {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.secondary_palette, 0.0, 98.0)
                } else {
                    Self::t_max_c_default(&scheme.secondary_palette)
                }
            }
            Variant::Vibrant => Self::t_max_c_bounds(&scheme.secondary_palette, 0.0, if scheme.is_dark { 90.0 } else { 98.0 }),
            _ => if scheme.is_dark { 80.0 } else { Self::t_max_c_default(&scheme.secondary_palette) },
        }
    }

    fn secondary_dim_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.variant == Variant::Neutral {
            85.0
        } else {
            Self::t_max_c_bounds(&scheme.secondary_palette, 0.0, 90.0)
        }
    }

    fn secondary_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            return 30.0;
        }
        match scheme.variant {
            Variant::Vibrant => {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.secondary_palette, 30.0, 40.0)
                } else {
                    Self::t_max_c_bounds(&scheme.secondary_palette, 84.0, 90.0)
                }
            }
            Variant::Expressive => if scheme.is_dark { 15.0 } else { Self::t_max_c_bounds(&scheme.secondary_palette, 90.0, 95.0) },
            _ => if scheme.is_dark { 25.0 } else { 90.0 },
        }
    }

    fn tertiary_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            if scheme.variant == Variant::TonalSpot {
                return Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 90.0);
            }
            return Self::t_max_c_default(&scheme.tertiary_palette);
        }
        match scheme.variant {
            Variant::Expressive | Variant::Vibrant => {
                let upper = if Hct::is_cyan(scheme.tertiary_palette.hue) { 88.0 } else if scheme.is_dark { 98.0 } else { 100.0 };
                Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, upper)
            }
            _ => { // Neutral & TonalSpot
                if scheme.is_dark {
                    Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 98.0)
                } else {
                    Self::t_max_c_default(&scheme.tertiary_palette)
                }
            }
        }
    }

    fn tertiary_dim_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.variant == Variant::TonalSpot {
            Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 90.0)
        } else {
            Self::t_max_c_default(&scheme.tertiary_palette)
        }
    }

    fn tertiary_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            if scheme.variant == Variant::TonalSpot {
                return Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 90.0);
            }
            return Self::t_max_c_default(&scheme.tertiary_palette);
        }
        match scheme.variant {
            Variant::Neutral => if scheme.is_dark {
                Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 93.0)
            } else {
                Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 96.0)
            },
            Variant::TonalSpot => Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, if scheme.is_dark { 93.0 } else { 100.0 }),
            Variant::Expressive => {
                let upper = if Hct::is_cyan(scheme.tertiary_palette.hue) { 88.0 } else if scheme.is_dark { 93.0 } else { 100.0 };
                Self::t_max_c_bounds(&scheme.tertiary_palette, 75.0, upper)
            }
            _ => { // Vibrant
                if scheme.is_dark {
                    Self::t_max_c_bounds(&scheme.tertiary_palette, 0.0, 93.0)
                } else {
                    Self::t_max_c_bounds(&scheme.tertiary_palette, 72.0, 100.0)
                }
            }
        }
    }

    fn error_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Phone {
            if scheme.is_dark {
                Self::t_min_c(&scheme.error_palette, 0.0, 98.0)
            } else {
                Self::t_max_c_default(&scheme.error_palette)
            }
        } else {
            Self::t_min_c_default(&scheme.error_palette)
        }
    }

    fn error_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::t_min_c_default(&scheme.error_palette)
    }

    fn error_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Watch {
            30.0
        } else if scheme.is_dark {
            Self::t_min_c(&scheme.error_palette, 30.0, 93.0)
        } else {
            Self::t_max_c_bounds(&scheme.error_palette, 0.0, 90.0)
        }
    }

    fn primary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::primary_container_tone(&temp_s)
    }

    fn primary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::primary_fixed_tone(scheme)
    }

    fn secondary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::secondary_container_tone(&temp_s)
    }

    fn secondary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::secondary_fixed_tone(scheme)
    }

    fn tertiary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::tertiary_container_tone(&temp_s)
    }

    fn tertiary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::tertiary_fixed_tone(scheme)
    }
}

impl ColorSpec for ColorSpec2025 {

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Un-overridden / Inherited Properties
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn primary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.primary_palette_key_color()
    }

    fn secondary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.secondary_palette_key_color()
    }

    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.tertiary_palette_key_color()
    }

    fn neutral_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.neutral_palette_key_color()
    }

    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.neutral_variant_palette_key_color()
    }

    fn error_palette_key_color(&self) -> Arc<DynamicColor> {
        self.base.error_palette_key_color()
    }

    fn shadow(&self) -> Arc<DynamicColor> {
        self.base.shadow()
    }

    fn scrim(&self) -> Arc<DynamicColor> {
        self.base.scrim()
    }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        if scheme.is_dark {
            self.surface_bright()
        } else {
            self.surface_dim()
        }
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Surfaces [S]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn background(&self) -> Arc<DynamicColor> {
        let surface = self.surface();
        let color2025 = DynamicColor::new(
            "background".to_string(),
            surface.palette.clone(),
            surface.is_background,
            surface.chroma_multiplier.clone(),
            surface.background.clone(),
            Some(surface.tone.clone()),
            surface.second_background.clone(),
            surface.contrast_curve.clone(),
            surface.tone_delta_pair.clone(),
            surface.opacity.clone(),
        );
        self.base.background().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        let on_surface = self.on_surface();
        let os_tone = on_surface.tone.clone();
        let color2025 = DynamicColor::new(
            "on_background".to_string(),
            on_surface.palette.clone(),
            on_surface.is_background,
            on_surface.chroma_multiplier.clone(),
            on_surface.background.clone(),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Watch { 100.0 } else { os_tone(scheme) }
            })),
            on_surface.second_background.clone(),
            on_surface.contrast_curve.clone(),
            on_surface.tone_delta_pair.clone(),
            on_surface.opacity.clone(),
        );
        self.base.on_background().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None, None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark {
                        4.0
                    } else if Hct::is_yellow(s.neutral_palette.hue) {
                        99.0
                    } else if s.variant == Variant::Vibrant {
                        97.0
                    } else {
                        98.0
                    }
                } else {
                    0.0
                }
            })),
            None, None, None, None,
        );
        self.base.surface().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_dim".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if !s.is_dark {
                    match s.variant {
                        Variant::Neutral => 2.5,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 2.7 } else { 1.75 },
                        Variant::Vibrant => 1.36,
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    4.0
                } else if Hct::is_yellow(s.neutral_palette.hue) {
                    90.0
                } else if s.variant == Variant::Vibrant {
                    85.0
                } else {
                    87.0
                }
            })),
            None, None, None, None,
        );
        self.base.surface_dim().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_bright".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.is_dark {
                    match s.variant {
                        Variant::Neutral => 2.5,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 2.7 } else { 1.75 },
                        Variant::Vibrant => 1.36,
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    18.0
                } else if Hct::is_yellow(s.neutral_palette.hue) {
                    99.0
                } else if s.variant == Variant::Vibrant {
                    97.0
                } else {
                    98.0
                }
            })),
            None, None, None, None,
        );
        self.base.surface_bright().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_lowest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None, None,
            Some(Arc::new(|s| if s.is_dark { 0.0 } else { 100.0 })),
            None, None, None, None,
        );
        self.base.surface_container_lowest().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_low".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 1.3,
                        Variant::TonalSpot => 1.25,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 1.3 } else { 1.15 },
                        Variant::Vibrant => 1.08,
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark { 6.0 } else if Hct::is_yellow(s.neutral_palette.hue) { 98.0 } else if s.variant == Variant::Vibrant { 95.0 } else { 96.0 }
                } else { 15.0 }
            })),
            None, None, None, None,
        );
        self.base.surface_container_low().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 1.6,
                        Variant::TonalSpot => 1.4,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 1.6 } else { 1.3 },
                        Variant::Vibrant => 1.15,
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark { 9.0 } else if Hct::is_yellow(s.neutral_palette.hue) { 96.0 } else if s.variant == Variant::Vibrant { 92.0 } else { 94.0 }
                } else { 20.0 }
            })),
            None, None, None, None,
        );
        self.base.surface_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_high".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 1.9,
                        Variant::TonalSpot => 1.5,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 1.95 } else { 1.45 },
                        Variant::Vibrant => 1.22,
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark { 12.0 } else if Hct::is_yellow(s.neutral_palette.hue) { 94.0 } else if s.variant == Variant::Vibrant { 90.0 } else { 92.0 }
                } else { 25.0 }
            })),
            None, None, None, None,
        );
        self.base.surface_container_high().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_highest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                match s.variant {
                    Variant::Neutral => 2.2,
                    Variant::TonalSpot => 1.7,
                    Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { 2.3 } else { 1.6 },
                    Variant::Vibrant => 1.29,
                    _ => 1.0,
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    15.0
                } else if Hct::is_yellow(s.neutral_palette.hue) {
                    92.0
                } else if s.variant == Variant::Vibrant {
                    88.0
                } else {
                    90.0
                }
            })),
            None, None, None, None,
        );
        self.base.surface_container_highest().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let bg_func_for_tone = bg_func.clone();

        let color2025 = DynamicColor::new(
            "on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { if s.is_dark { 3.0 } else { 2.3 } } else { 1.6 },
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            Some(bg_func),
            Some(Arc::new(move |s| {
                if s.variant == Variant::Vibrant {
                    Self::t_max_c(&s.neutral_palette, 0.0, 100.0, 1.1)
                } else {
                    bg_func_for_tone(s).unwrap().get_tone(s)
                }
            })),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.is_dark && s.platform == Platform::Phone { 11.0 } else { 9.0 })))),
            None, None,
        );
        self.base.on_surface().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        let sch = self.surface_container_highest();
        let color2025 = DynamicColor::new(
            "surface_variant".to_string(),
            sch.palette.clone(),
            sch.is_background,
            sch.chroma_multiplier.clone(),
            sch.background.clone(),
            Some(sch.tone.clone()),
            sch.second_background.clone(),
            sch.contrast_curve.clone(),
            sch.tone_delta_pair.clone(),
            sch.opacity.clone(),
        );
        self.base.surface_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let color2025 = DynamicColor::new(
            "on_surface_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { if s.is_dark { 3.0 } else { 2.3 } } else { 1.6 },
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            Some(bg_func),
            None,
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { if s.is_dark { 6.0 } else { 4.5 } } else { 7.0 })))),
            None, None,
        );
        self.base.on_surface_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "inverse_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None, None,
            Some(Arc::new(|s| if s.is_dark { 98.0 } else { 4.0 })),
            None, None, None, None,
        );
        self.base.inverse_surface().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let inv_surface = self.inverse_surface();
        let color2025 = DynamicColor::new(
            "inverse_on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(inv_surface.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.inverse_on_surface().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let color2025 = DynamicColor::new(
            "outline".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { if s.is_dark { 3.0 } else { 2.3 } } else { 1.6 },
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 3.0 } else { 4.5 })))),
            None, None,
        );
        self.base.outline().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let color2025 = DynamicColor::new(
            "outline_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => if Hct::is_yellow(s.neutral_palette.hue) { if s.is_dark { 3.0 } else { 2.3 } } else { 1.6 },
                        _ => 1.0,
                    }
                } else { 1.0 }
            })),
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 1.5 } else { 3.0 })))),
            None, None,
        );
        self.base.outline_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_tint(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let color2025 = DynamicColor::new(
            "surface_tint".to_string(),
            primary.palette.clone(),
            primary.is_background,
            primary.chroma_multiplier.clone(),
            primary.background.clone(),
            Some(primary.tone.clone()),
            primary.second_background.clone(),
            primary.contrast_curve.clone(),
            primary.tone_delta_pair.clone(),
            primary.opacity.clone(),
        );
        self.base.surface_tint().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Primaries [P]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn primary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let pc_stub = Arc::new(DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_container_tone)),
            None, None, None, None
        ));

        let p_stub = Arc::new(DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(ToneDeltaPair::new(
                    pc_stub.clone(),
                    p_stub.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::primary_tone)),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 4.5 } else { 7.0 })))),
            Some(tdp),
            None,
        );
        self.base.primary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();

        let pd_stub = Arc::new(DynamicColor::new(
            "primary_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_dim_tone)),
            None, None, None, None
        ));

        let p_stub = Arc::new(DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                pd_stub.clone(),
                p_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Farther,
            ))
        });

        Some(Arc::new(DynamicColor::new(
            "primary_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(Self::primary_dim_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(tdp),
            None,
        )))
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let primary_dim = self.primary_dim().unwrap();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone { primary.clone() } else { primary_dim.clone() })
        });

        let color2025 = DynamicColor::new(
            "on_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_primary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let pc_stub = Arc::new(DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_container_tone)),
            None, None, None, None
        ));

        let pd_stub = Arc::new(DynamicColor::new(
            "primary_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_dim_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Watch {
                Some(ToneDeltaPair::new(
                    pc_stub.clone(),
                    pd_stub.clone(),
                    10.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::primary_container_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.primary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let pc = self.primary_container();
        let color2025 = DynamicColor::new(
            "on_primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pc.clone()))),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_primary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        let inv_surface = self.inverse_surface();
        let color2025 = DynamicColor::new(
            "inverse_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(inv_surface.clone()))),
            Some(Arc::new(|s| Self::t_max_c_default(&s.primary_palette))),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.inverse_primary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Secondaries [Q]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn secondary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let sc_stub = Arc::new(DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_container_tone)),
            None, None, None, None
        ));

        let s_stub = Arc::new(DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(ToneDeltaPair::new(
                    sc_stub.clone(),
                    s_stub.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::secondary_tone)),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 4.5 } else { 7.0 })))),
            Some(tdp),
            None,
        );
        self.base.secondary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();

        let sd_stub = Arc::new(DynamicColor::new(
            "secondary_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_dim_tone)),
            None, None, None, None
        ));

        let s_stub = Arc::new(DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                sd_stub.clone(),
                s_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Farther,
            ))
        });

        Some(Arc::new(DynamicColor::new(
            "secondary_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(Self::secondary_dim_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(tdp),
            None,
        )))
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let secondary = self.secondary();
        let secondary_dim = self.secondary_dim().unwrap();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone { secondary.clone() } else { secondary_dim.clone() })
        });

        let color2025 = DynamicColor::new(
            "on_secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_secondary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let sc_stub = Arc::new(DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_container_tone)),
            None, None, None, None
        ));

        let sd_stub = Arc::new(DynamicColor::new(
            "secondary_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_dim_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Watch {
                Some(ToneDeltaPair::new(
                    sc_stub.clone(),
                    sd_stub.clone(),
                    10.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::secondary_container_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.secondary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let sc = self.secondary_container();
        let color2025 = DynamicColor::new(
            "on_secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sc.clone()))),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_secondary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Tertiaries [T]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn tertiary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let tc_stub = Arc::new(DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_container_tone)),
            None, None, None, None
        ));

        let t_stub = Arc::new(DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(ToneDeltaPair::new(
                    tc_stub.clone(),
                    t_stub.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::tertiary_tone)),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 4.5 } else { 7.0 })))),
            Some(tdp),
            None,
        );
        self.base.tertiary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();

        let td_stub = Arc::new(DynamicColor::new(
            "tertiary_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_dim_tone)),
            None, None, None, None
        ));

        let t_stub = Arc::new(DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                td_stub.clone(),
                t_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Farther,
            ))
        });

        Some(Arc::new(DynamicColor::new(
            "tertiary_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(Self::tertiary_dim_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(tdp),
            None,
        )))
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let tertiary = self.tertiary();
        let tertiary_dim = self.tertiary_dim().unwrap();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone { tertiary.clone() } else { tertiary_dim.clone() })
        });

        let color2025 = DynamicColor::new(
            "on_tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_tertiary().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let tc_stub = Arc::new(DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_container_tone)),
            None, None, None, None
        ));

        let td_stub = Arc::new(DynamicColor::new(
            "tertiary_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_dim_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Watch {
                Some(ToneDeltaPair::new(
                    tc_stub.clone(),
                    td_stub.clone(),
                    10.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::tertiary_container_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.tertiary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let tc = self.tertiary_container();
        let color2025 = DynamicColor::new(
            "on_tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tc.clone()))),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_tertiary_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Errors [E]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn error(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone {
                if s.is_dark { surface_bright.clone() } else { surface_dim.clone() }
            } else {
                surface_container_high.clone()
            })
        });

        let ec_stub = Arc::new(DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_container_tone)),
            None, None, None, None
        ));

        let e_stub = Arc::new(DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(ToneDeltaPair::new(
                    ec_stub.clone(),
                    e_stub.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::error_tone)),
            None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 4.5 } else { 7.0 })))),
            Some(tdp),
            None,
        );
        self.base.error().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();

        let ed_stub = Arc::new(DynamicColor::new(
            "error_dim".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_dim_tone)),
            None, None, None, None
        ));

        let e_stub = Arc::new(DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                ed_stub.clone(),
                e_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Farther,
            ))
        });

        Some(Arc::new(DynamicColor::new(
            "error_dim".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(Self::error_dim_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(tdp),
            None,
        )))
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let error = self.error();
        let error_dim = self.error_dim().unwrap();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            Some(if s.platform == Platform::Phone { error.clone() } else { error_dim.clone() })
        });

        let color2025 = DynamicColor::new(
            "on_error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false, None,
            Some(bg_func),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 6.0 } else { 7.0 })))),
            None, None,
        );
        self.base.on_error().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let ec_stub = Arc::new(DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_container_tone)),
            None, None, None, None
        ));

        let ed_stub = Arc::new(DynamicColor::new(
            "error_dim".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::error_dim_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |s| {
            if s.platform == Platform::Watch {
                Some(ToneDeltaPair::new(
                    ec_stub.clone(),
                    ed_stub.clone(),
                    10.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::error_container_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.error_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let ec = self.error_container();
        let color2025 = DynamicColor::new(
            "on_error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(ec.clone()))),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.platform == Platform::Phone { 4.5 } else { 7.0 })))),
            None, None,
        );
        self.base.on_error_container().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Primary Fixed Colors [PF]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::primary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.primary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let pfd_stub = Arc::new(DynamicColor::new(
            "primary_fixed_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_fixed_dim_tone)),
            None, None, None, None
        ));

        let pf_stub = Arc::new(DynamicColor::new(
            "primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_fixed_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                pfd_stub.clone(),
                pf_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Exact,
            ))
        });

        let color2025 = DynamicColor::new(
            "primary_fixed_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::primary_fixed_dim_tone)),
            None, None,
            Some(tdp),
            None,
        );
        self.base.primary_fixed_dim().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_primary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_primary_fixed_variant".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_primary_fixed_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Secondary Fixed Colors [QF]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::secondary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.secondary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let sfd_stub = Arc::new(DynamicColor::new(
            "secondary_fixed_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_fixed_dim_tone)),
            None, None, None, None
        ));

        let sf_stub = Arc::new(DynamicColor::new(
            "secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_fixed_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                sfd_stub.clone(),
                sf_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Exact,
            ))
        });

        let color2025 = DynamicColor::new(
            "secondary_fixed_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::secondary_fixed_dim_tone)),
            None, None,
            Some(tdp),
            None,
        );
        self.base.secondary_fixed_dim().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_secondary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_secondary_fixed_variant".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_secondary_fixed_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Tertiary Fixed Colors [TF]
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();

        let bg_func: DynamicColorFunction<Option<Arc<DynamicColor>>> = Arc::new(move |s| {
            if s.platform == Platform::Phone {
                Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
            } else { None }
        });

        let color2025 = DynamicColor::new(
            "tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(bg_func),
            Some(Arc::new(Self::tertiary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.platform == Platform::Phone && s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.tertiary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let tfd_stub = Arc::new(DynamicColor::new(
            "tertiary_fixed_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_fixed_dim_tone)),
            None, None, None, None
        ));

        let tf_stub = Arc::new(DynamicColor::new(
            "tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_fixed_tone)),
            None, None, None, None
        ));

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                tfd_stub.clone(),
                tf_stub.clone(),
                5.0,
                TonePolarity::Darker,
                true,
                DeltaConstraint::Exact,
            ))
        });

        let color2025 = DynamicColor::new(
            "tertiary_fixed_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None, None,
            Some(Arc::new(Self::tertiary_fixed_dim_tone)),
            None, None,
            Some(tdp),
            None,
        );
        self.base.tertiary_fixed_dim().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_tertiary_fixed().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim();
        let color2025 = DynamicColor::new(
            "on_tertiary_fixed_variant".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_tertiary_fixed_variant().extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Color value calculations
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        let palette = (color.palette)(scheme);
        let tone = self.get_tone(scheme, color);
        let hue = palette.hue;
        let chroma_multiplier = color.chroma_multiplier.as_ref().map_or(1.0, |f| f(scheme));
        let chroma = palette.chroma * chroma_multiplier;
        Hct::from(hue, chroma, tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        let tone_delta_pair = color.tone_delta_pair.as_ref().and_then(|f| f(scheme));

        // Case 0: Tone delta pair.
        if let Some(tdp) = tone_delta_pair {
            let role_a = &tdp.role_a;
            let role_b = &tdp.role_b;
            let polarity = tdp.polarity;
            let constraint = tdp.constraint;
            let absolute_delta = if polarity == TonePolarity::Darker
                || (polarity == TonePolarity::RelativeLighter && scheme.is_dark)
                || (polarity == TonePolarity::RelativeDarker && !scheme.is_dark)
            {
                -tdp.delta
            } else {
                tdp.delta
            };

            let am_role_a = color.name == role_a.name;
            let self_role = if am_role_a { role_a } else { role_b };
            let reference_role = if am_role_a { role_b } else { role_a };

            let mut self_tone = (self_role.tone)(scheme);
            let reference_tone = reference_role.get_tone(scheme);
            let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };

            match constraint {
                DeltaConstraint::Exact => self_tone = (reference_tone + relative_delta).clamp(0.0, 100.0),
                DeltaConstraint::Nearer => {
                    if relative_delta > 0.0 {
                        self_tone = self_tone.clamp(reference_tone, reference_tone + relative_delta).clamp(0.0, 100.0);
                    } else {
                        self_tone = self_tone.clamp(reference_tone + relative_delta, reference_tone).clamp(0.0, 100.0);
                    }
                }
                DeltaConstraint::Farther => {
                    if relative_delta > 0.0 {
                        if(reference_tone + relative_delta > 100.){
                            println!("oeps")
                        }
                        self_tone = self_tone.clamp(reference_tone + relative_delta, 100.0);
                    } else {
                        self_tone = self_tone.clamp(0.0, reference_tone + relative_delta);
                    }
                }
            }

            if let (Some(bg_fn), Some(cc_fn)) = (color.background.as_ref(), color.contrast_curve.as_ref()) {
                if let (Some(bg), Some(cc)) = (bg_fn(scheme), cc_fn(scheme)) {
                    let bg_tone = bg.get_tone(scheme);
                    let self_contrast = cc.get(scheme.contrast_level);
                    if Contrast::ratio_of_tones(bg_tone, self_tone) >= self_contrast && scheme.contrast_level >= 0.0 {
                        // Keep self_tone
                    } else {
                        self_tone = DynamicColor::foreground_tone(bg_tone, self_contrast);
                    }
                }
            }

            if color.is_background && !color.name.ends_with("_fixed_dim") {
                if self_tone >= 57.0 {
                    self_tone = self_tone.clamp(65.0, 100.0);
                } else {
                    self_tone = self_tone.clamp(0.0, 49.0);
                }
            }
            return self_tone;
        } else {
            // Case 1: No tone delta pair; just solve for itself.
            let mut answer = (color.tone)(scheme);

            let background = color.background.as_ref().and_then(|f| f(scheme));
            let contrast_curve = color.contrast_curve.as_ref().and_then(|f| f(scheme));

            let (Some(bg_color), Some(cc)) = (background, contrast_curve) else {
                return answer;
            };

            let bg_tone = bg_color.get_tone(scheme);
            let desired_ratio = cc.get(scheme.contrast_level);

            if Contrast::ratio_of_tones(bg_tone, answer) >= desired_ratio && scheme.contrast_level >= 0.0 {
                // keep answer
            } else {
                answer = DynamicColor::foreground_tone(bg_tone, desired_ratio);
            }

            if color.is_background && !color.name.ends_with("_fixed_dim") {
                if answer >= 57.0 {
                    answer = answer.clamp(65.0, 100.0);
                } else {
                    answer = answer.clamp(0.0, 49.0);
                }
            }

            let second_background = color.second_background.as_ref().and_then(|f| f(scheme));
            let Some(bg2_color) = second_background else {
                return answer;
            };

            // Case 2: Adjust for dual backgrounds.
            let bg_tone1 = bg_color.get_tone(scheme);
            let bg_tone2 = bg2_color.get_tone(scheme);
            let upper = bg_tone1.max(bg_tone2);
            let lower = bg_tone1.min(bg_tone2);

            if Contrast::ratio_of_tones(upper, answer) >= desired_ratio
                && Contrast::ratio_of_tones(lower, answer) >= desired_ratio
            {
                return answer;
            }

            let light_option = Contrast::lighter(upper, desired_ratio);
            let dark_option = Contrast::darker(lower, desired_ratio);

            let prefers_light = DynamicColor::tone_prefers_light_foreground(bg_tone1)
                || DynamicColor::tone_prefers_light_foreground(bg_tone2);

            if prefers_light {
                return light_option.unwrap_or(100.0);
            }

            match (light_option, dark_option) {
                (Some(_l), Some(d)) => d,
                (Some(l), None) => l,
                (None, Some(d)) => d,
                (None, None) => 0.0,
            }
        }
    }

    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    // Scheme Palettes
    // â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn get_primary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    if Hct::is_blue(source_color_hct.hue()) { 12.0 } else { 8.0 }
                } else if Hct::is_blue(source_color_hct.hue()) { 16.0 } else { 12.0 }
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone && is_dark { 26.0 } else { 32.0 }
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    if is_dark { 36.0 } else { 48.0 }
                } else { 40.0 }
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone { 74.0 } else { 56.0 }
            ),
            _ => self.base.get_primary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }

    fn get_secondary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    if Hct::is_blue(source_color_hct.hue()) { 6.0 } else { 4.0 }
                } else if Hct::is_blue(source_color_hct.hue()) { 10.0 } else { 6.0 }
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(source_color_hct.hue(), 16.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-160.0, 155.0, -100.0, 96.0, -96.0, -156.0, -165.0, -160.0],
                ),
                if platform == Platform::Phone { if is_dark { 16.0 } else { 24.0 } } else { 24.0 }
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                    &[-14.0, 10.0, -14.0, 10.0, -14.0],
                ),
                if platform == Platform::Phone { 56.0 } else { 36.0 }
            ),
            _ => self.base.get_secondary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }

    fn get_tertiary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 105.0, 161.0, 204.0, 278.0, 333.0, 360.0],
                    &[-32.0, 26.0, 10.0, -39.0, 24.0, -15.0, -32.0],
                ),
                if platform == Platform::Phone { 20.0 } else { 36.0 }
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 20.0, 71.0, 161.0, 333.0, 360.0],
                    &[-40.0, 48.0, -32.0, 40.0, -32.0],
                ),
                if platform == Platform::Phone { 28.0 } else { 32.0 }
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-165.0, 160.0, -105.0, 101.0, -101.0, -160.0, -170.0, -165.0],
                ),
                48.0
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 71.0, 105.0, 140.0, 161.0, 253.0, 333.0, 360.0],
                    &[-72.0, 35.0, 24.0, -24.0, 62.0, 50.0, 62.0, -72.0],
                ),
                56.0
            ),
            _ => self.base.get_tertiary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }

    fn get_neutral_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone { 1.4 } else { 6.0 }
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone { 5.0 } else { 10.0 }
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                Self::get_expressive_neutral_hue(source_color_hct),
                Self::get_expressive_neutral_chroma(source_color_hct, is_dark, platform)
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                Self::get_vibrant_neutral_hue(source_color_hct),
                Self::get_vibrant_neutral_chroma(source_color_hct, platform)
            ),
            _ => self.base.get_neutral_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }

    fn get_neutral_variant_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                (if platform == Platform::Phone { 1.4 } else { 6.0 }) * 2.2
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                (if platform == Platform::Phone { 5.0 } else { 10.0 }) * 1.7
            ),
            Variant::Expressive => {
                let hue = Self::get_expressive_neutral_hue(source_color_hct);
                let chroma = Self::get_expressive_neutral_chroma(source_color_hct, is_dark, platform);
                TonalPalette::from_hue_and_chroma(hue, chroma * if hue >= 105.0 && hue < 125.0 { 1.6 } else { 2.3 })
            }
            Variant::Vibrant => {
                let hue = Self::get_vibrant_neutral_hue(source_color_hct);
                let chroma = Self::get_vibrant_neutral_chroma(source_color_hct, platform);
                TonalPalette::from_hue_and_chroma(hue, chroma * 1.29)
            }
            _ => self.base.get_neutral_variant_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }

    fn get_error_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        let error_hue = DynamicScheme::get_piecewise_value(
            source_color_hct,
            &[0.0, 3.0, 13.0, 23.0, 33.0, 43.0, 153.0, 273.0, 360.0],
            &[12.0, 22.0, 32.0, 12.0, 22.0, 32.0, 22.0, 12.0],
        );
        match variant {
            Variant::Neutral => TonalPalette::from_hue_and_chroma(error_hue, if platform == Platform::Phone { 50.0 } else { 40.0 }),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(error_hue, if platform == Platform::Phone { 60.0 } else { 48.0 }),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(error_hue, if platform == Platform::Phone { 64.0 } else { 48.0 }),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(error_hue, if platform == Platform::Phone { 80.0 } else { 60.0 }),
            _ => self.base.get_error_palette(variant, source_color_hct, is_dark, platform, contrast_level)
        }
    }
}