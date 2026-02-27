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
use crate::dislike::dislike_analyzer::DislikeAnalyzer;
use crate::dynamic::color_spec::{ColorSpec, Platform, SpecVersion};
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::temperature::temperature_cache::TemperatureCache;
use crate::utils::math_utils::MathUtils;

pub struct ColorSpec2021 {
    override_spec: SpecVersion,
}

impl ColorSpec2021 {
    #[must_use]
    pub const fn with_override_spec(override_spec: SpecVersion) -> Self {
        Self { override_spec }
    }

    #[must_use]
    pub const fn new() -> Self {
        Self {
            override_spec: SpecVersion::Spec2021,
        }
    }

    fn is_fidelity(scheme: &DynamicScheme) -> bool {
        scheme.variant == Variant::Fidelity || scheme.variant == Variant::Content
    }

    fn is_monochrome(scheme: &DynamicScheme) -> bool {
        scheme.variant == Variant::Monochrome
    }

    fn find_desired_chroma_by_tone(
        hue: f64,
        chroma: f64,
        tone: f64,
        by_decreasing_tone: bool,
    ) -> f64 {
        let mut answer = tone;
        let mut closest_to_chroma = Hct::from(hue, chroma, tone);
        if closest_to_chroma.chroma() < chroma {
            let mut chroma_peak = closest_to_chroma.chroma();
            while closest_to_chroma.chroma() < chroma {
                answer += if by_decreasing_tone { -1.0 } else { 1.0 };
                let potential_solution = Hct::from(hue, chroma, answer);
                if chroma_peak > potential_solution.chroma() {
                    break;
                }
                if (potential_solution.chroma() - chroma).abs() < 0.4 {
                    break;
                }
                let potential_delta = (potential_solution.chroma() - chroma).abs();
                let current_delta = (closest_to_chroma.chroma() - chroma).abs();
                if potential_delta < current_delta {
                    closest_to_chroma = potential_solution;
                }
                chroma_peak = chroma_peak.max(potential_solution.chroma());
            }
        }
        answer
    }
}

impl Default for ColorSpec2021 {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpec for ColorSpec2021 {
    // ————————————————————————————————————————————————————————————————
    // Main Palette Key Colors
    // ————————————————————————————————————————————————————————————————

    fn primary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary_palette_key_color".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.primary_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    fn secondary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary_palette_key_color".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.secondary_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary_palette_key_color".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.tertiary_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    fn neutral_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "neutral_palette_key_color".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.neutral_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "neutral_variant_palette_key_color".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.neutral_variant_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    fn error_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "error_palette_key_color".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|s| s.error_palette.key_color.tone())),
            None,
            None,
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Surfaces [S]
    // ————————————————————————————————————————————————————————————————

    fn background(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "background".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        let override_spec = self.override_spec;
        Arc::new(DynamicColor::new(
            "on_background".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(move |s| {
                Some(ColorSpecs::get(override_spec).background())
            })),
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 3.0, 4.5, 7.0)))),
            None,
            None,
        ))
    }

    fn surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_dim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    6.0
                } else {
                    ContrastCurve::new(87.0, 87.0, 80.0, 75.0).get(s.contrast_level)
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_bright".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(24.0, 24.0, 29.0, 34.0).get(s.contrast_level)
                } else {
                    98.0
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_lowest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(4.0, 4.0, 2.0, 0.0).get(s.contrast_level)
                } else {
                    100.0
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_low".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(10.0, 10.0, 11.0, 12.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(96.0, 96.0, 96.0, 95.0).get(s.contrast_level)
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(12.0, 12.0, 16.0, 20.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(94.0, 94.0, 92.0, 90.0).get(s.contrast_level)
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_high".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(17.0, 17.0, 21.0, 25.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(92.0, 92.0, 88.0, 85.0).get(s.contrast_level)
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_highest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(22.0, 22.0, 26.0, 30.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(90.0, 90.0, 84.0, 80.0).get(s.contrast_level)
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_surface_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 30.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "inverse_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 20.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "inverse_on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).inverse_surface())
            })),
            Some(Arc::new(|s| if s.is_dark { 20.0 } else { 95.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn outline(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "outline".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 60.0 } else { 50.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.5, 3.0, 4.5, 7.0)))),
            None,
            None,
        ))
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "outline_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 80.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None,
            None,
        ))
    }

    fn shadow(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "shadow".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|_| 0.0)),
            None,
            None,
            None,
            None,
        ))
    }

    fn scrim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "scrim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            None,
            Some(Arc::new(|_| 0.0)),
            None,
            None,
            None,
            None,
        ))
    }

    fn surface_tint(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_tint".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Primaries [P]
    // ————————————————————————————————————————————————————————————————

    fn primary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 100.0 } else { 0.0 }
                } else if s.is_dark {
                    80.0
                } else {
                    40.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.primary_container(),
                    spec.primary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        None
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary())
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 90.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if Self::is_fidelity(s) {
                    s.source_color_hct().tone()
                } else if Self::is_monochrome(s) {
                    if s.is_dark { 85.0 } else { 25.0 }
                } else if s.is_dark {
                    30.0
                } else {
                    90.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.primary_container(),
                    spec.primary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let override_spec = [self.override_spec; 1];
        Arc::new(DynamicColor::new(
            "on_primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |s| {
                Some(ColorSpecs::get(override_spec[0]).primary_container())
            })),
            Some(Arc::new(move |s| {
                if Self::is_fidelity(s) {
                    // make sure it uses the tone getter from DynamicColor, not the get_tone function on ColorSpec
                    let pc = ColorSpecs::get(override_spec[0]).primary_container();
                    let pc_raw_tone = (pc.tone)(s);
                    DynamicColor::foreground_tone(pc_raw_tone, 4.5)
                } else if Self::is_monochrome(s) {
                    if s.is_dark { 0.0 } else { 100.0 }
                } else {
                    if s.is_dark { 90.0 } else { 30.0 }
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "inverse_primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).inverse_surface())
            })),
            Some(Arc::new(|s| if s.is_dark { 40.0 } else { 80.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Secondaries [Q]
    // ————————————————————————————————————————————————————————————————

    fn secondary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.secondary_container(),
                    spec.secondary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        None
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary())
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 100.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                let initial = if s.is_dark { 30.0 } else { 90.0 };
                if Self::is_monochrome(s) {
                    if s.is_dark { 30.0 } else { 85.0 }
                } else if !Self::is_fidelity(s) {
                    initial
                } else {
                    Self::find_desired_chroma_by_tone(
                        s.secondary_palette.hue,
                        s.secondary_palette.chroma,
                        initial,
                        !s.is_dark,
                    )
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.secondary_container(),
                    spec.secondary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let override_spec = [self.override_spec; 1];
        Arc::new(DynamicColor::new(
            "on_secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_container())
            })),
            Some(Arc::new(move |s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 10.0 }
                } else if !Self::is_fidelity(s) {
                    if s.is_dark { 90.0 } else { 30.0 }
                } else {
                    let sc = ColorSpecs::get(override_spec[0]).secondary_container();
                    let sc_raw_tone = (sc.tone)(s);
                    DynamicColor::foreground_tone(sc_raw_tone, 4.5)
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Tertiaries [T]
    // ————————————————————————————————————————————————————————————————

    fn tertiary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 25.0 }
                } else if s.is_dark {
                    80.0
                } else {
                    40.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.tertiary_container(),
                    spec.tertiary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        None
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary())
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 10.0 } else { 90.0 }
                } else if s.is_dark {
                    20.0
                } else {
                    100.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 60.0 } else { 49.0 }
                } else if !Self::is_fidelity(s) {
                    if s.is_dark { 30.0 } else { 90.0 }
                } else {
                    let proposed = s.tertiary_palette.get_hct(s.source_color_hct().tone());
                    DislikeAnalyzer::fix_if_disliked(proposed).tone()
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.tertiary_container(),
                    spec.tertiary(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let override_spec = [self.override_spec; 1];
        Arc::new(DynamicColor::new(
            "on_tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_container())
            })),
            Some(Arc::new(move |s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 0.0 } else { 100.0 }
                } else if !Self::is_fidelity(s) {
                    if s.is_dark { 90.0 } else { 30.0 }
                } else {
                    let tc = ColorSpecs::get(override_spec[0]).tertiary_container();
                    let tc_raw_tone = (tc.tone)(s);
                    DynamicColor::foreground_tone(tc_raw_tone, 4.5)
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Errors [E]
    // ————————————————————————————————————————————————————————————————

    fn error(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "error".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.error_container(),
                    spec.error(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        None
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_error".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| Some(ColorSpecs::get(s.spec_version).error()))),
            Some(Arc::new(|s| if s.is_dark { 20.0 } else { 100.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.error_container(),
                    spec.error(),
                    10.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Nearer,
                ))
            })),
            None,
        ))
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).error_container())
            })),
            Some(Arc::new(|s| {
                if Self::is_monochrome(s) {
                    if s.is_dark { 90.0 } else { 10.0 }
                } else if s.is_dark {
                    90.0
                } else {
                    30.0
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Fixed colors
    // ————————————————————————————————————————————————————————————————

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary_fixed".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 40.0 } else { 90.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.primary_fixed(),
                    spec.primary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary_fixed_dim".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 30.0 } else { 80.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.primary_fixed(),
                    spec.primary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_primary_fixed".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed_dim())
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 100.0 } else { 10.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_primary_fixed_variant".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed_dim())
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 90.0 } else { 30.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary_fixed".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 80.0 } else { 90.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.secondary_fixed(),
                    spec.secondary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary_fixed_dim".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 70.0 } else { 80.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.secondary_fixed(),
                    spec.secondary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_secondary_fixed".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed_dim())
            })),
            Some(Arc::new(|_| 10.0)),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_secondary_fixed_variant".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed_dim())
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 25.0 } else { 30.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary_fixed".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 40.0 } else { 90.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.tertiary_fixed(),
                    spec.tertiary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary_fixed_dim".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 30.0 } else { 80.0 },
            )),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.tertiary_fixed(),
                    spec.tertiary_fixed_dim(),
                    10.0,
                    TonePolarity::Lighter,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        ))
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_tertiary_fixed".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed_dim())
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 100.0 } else { 10.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None,
            None,
        ))
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "on_tertiary_fixed_variant".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed_dim())
            })),
            Some(Arc::new(
                |s| if Self::is_monochrome(s) { 90.0 } else { 30.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed())
            })),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None,
            None,
        ))
    }

    // ————————————————————————————————————————————————————————————————
    // Calculations & Palettes
    // ————————————————————————————————————————————————————————————————

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        if scheme.is_dark {
            self.surface_bright()
        } else {
            self.surface_dim()
        }
    }

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        let tone = self.get_tone(scheme, color);
        // dbg!(&tone);
        (color.palette)(scheme).get_hct(tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        let decreasing_contrast = scheme.contrast_level < 0.0;
        let tone_delta_pair = color.tone_delta_pair.as_ref().and_then(|f| f(scheme));
        // dbg!(&tone_delta_pair.is_none());

        if let Some(tdp) = tone_delta_pair {
            let role_a = &tdp.role_a;
            let role_b = &tdp.role_b;
            let delta = tdp.delta;
            let polarity = tdp.polarity;
            let stay_together = tdp.stay_together;
            let a_is_nearer = tdp.constraint == DeltaConstraint::Nearer
                || (polarity == TonePolarity::Lighter && !scheme.is_dark)
                || (polarity == TonePolarity::Darker && !scheme.is_dark);
            let nearer = if a_is_nearer { role_a } else { role_b };
            let farther = if a_is_nearer { role_b } else { role_a };
            let am_nearer = color.name == nearer.name;
            let expansion_dir: f64 = if scheme.is_dark { 1.0 } else { -1.0 };
            let mut n_tone = (nearer.tone)(scheme);
            let mut f_tone = (farther.tone)(scheme);

            if let (Some(bg_fn), Some(n_cc), Some(f_cc)) = (
                color.background.as_ref(),
                nearer.contrast_curve.as_ref().and_then(|f| f(scheme)),
                farther.contrast_curve.as_ref().and_then(|f| f(scheme)),
            ) && let Some(bg_color) = bg_fn(scheme)
            {
                let n_contrast = n_cc.get(scheme.contrast_level);
                let f_contrast = f_cc.get(scheme.contrast_level);
                let bg_tone = bg_color.get_tone(scheme);
                if Contrast::ratio_of_tones(bg_tone, n_tone) < n_contrast {
                    n_tone = DynamicColor::foreground_tone(bg_tone, n_contrast);
                }
                if Contrast::ratio_of_tones(bg_tone, f_tone) < f_contrast {
                    f_tone = DynamicColor::foreground_tone(bg_tone, f_contrast);
                }
                if decreasing_contrast {
                    n_tone = DynamicColor::foreground_tone(bg_tone, n_contrast);
                    f_tone = DynamicColor::foreground_tone(bg_tone, f_contrast);
                }
            }

            if (f_tone - n_tone) * expansion_dir < delta {
                f_tone = delta.mul_add(expansion_dir, n_tone).clamp(0.0, 100.0);
                if (f_tone - n_tone) * expansion_dir < delta {
                    n_tone = delta.mul_add(-expansion_dir, f_tone).clamp(0.0, 100.0);
                }
            }

            if (50.0..60.0).contains(&n_tone) {
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = f_tone.max(delta.mul_add(expansion_dir, n_tone));
                } else {
                    n_tone = 49.0;
                    f_tone = f_tone.min(delta.mul_add(expansion_dir, n_tone));
                }
            } else if (50.0..60.0).contains(&f_tone) {
                if stay_together {
                    if expansion_dir > 0.0 {
                        n_tone = 60.0;
                        f_tone = f_tone.max(delta.mul_add(expansion_dir, n_tone));
                    } else {
                        n_tone = 49.0;
                        f_tone = f_tone.min(delta.mul_add(expansion_dir, n_tone));
                    }
                } else {
                    f_tone = if expansion_dir > 0.0 { 60.0 } else { 49.0 };
                }
            }
            if am_nearer { n_tone } else { f_tone }
        } else {
            let mut answer = (color.tone)(scheme);
            // dbg!(&answer);
            let background = color.background.as_ref().and_then(|f| f(scheme));
            let contrast_curve = color.contrast_curve.as_ref().and_then(|f| f(scheme));
            let (Some(bg_color), Some(cc)) = (background, contrast_curve) else {
                return answer;
            };
            let bg_tone = bg_color.get_tone(scheme);
            let desired_ratio = cc.get(scheme.contrast_level);
            if Contrast::ratio_of_tones(bg_tone, answer) < desired_ratio {
                answer = DynamicColor::foreground_tone(bg_tone, desired_ratio);
            }
            if decreasing_contrast {
                answer = DynamicColor::foreground_tone(bg_tone, desired_ratio);
            }
            if color.is_background && (50.0..60.0).contains(&answer) {
                answer = if Contrast::ratio_of_tones(49.0, bg_tone) >= desired_ratio {
                    49.0
                } else {
                    60.0
                };
            }
            let second_bg = color.second_background.as_ref().and_then(|f| f(scheme));
            let Some(bg2_color) = second_bg else {
                return answer;
            };
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
            if DynamicColor::tone_prefers_light_foreground(bg_tone1)
                || DynamicColor::tone_prefers_light_foreground(bg_tone2)
            {
                return light_option.unwrap_or(100.0);
            }
            match (light_option, dark_option) {
                (Some(_), Some(d)) => d,
                (Some(l), None) => l,
                (None, Some(d)) => d,
                (None, None) => 0.0,
            }
        }
    }

    fn get_primary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        _: bool,
        _: Platform,
        _: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => {
                TonalPalette::from_hue_and_chroma(src.hue(), src.chroma())
            }
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() - 50.0),
                48.0,
            ),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 12.0),
            Variant::Rainbow => TonalPalette::from_hue_and_chroma(src.hue(), 48.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 36.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() + 240.0),
                40.0,
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(src.hue(), 200.0),
            v => panic!("{v:?} variant not supported in Spec2021"),
        }
    }

    fn get_secondary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        _: bool,
        _: Platform,
        _: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => TonalPalette::from_hue_and_chroma(
                src.hue(),
                (src.chroma() - 32.0).max(src.chroma() * 0.5),
            ),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() - 50.0),
                36.0,
            ),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 8.0),
            Variant::Rainbow | Variant::TonalSpot => {
                TonalPalette::from_hue_and_chroma(src.hue(), 16.0)
            }
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    src,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0],
                ),
                24.0,
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    src,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0],
                ),
                24.0,
            ),
            v => panic!("{v:?} variant not supported in Spec2021"),
        }
    }

    fn get_tertiary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        _: bool,
        _: Platform,
        _: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Content => TonalPalette::from_hct(DislikeAnalyzer::fix_if_disliked(
                TemperatureCache::new(*src).get_analogous_colors_with_options(3, 6)[2],
            )),
            Variant::Fidelity => TonalPalette::from_hct(DislikeAnalyzer::fix_if_disliked(
                TemperatureCache::new(*src).complement(),
            )),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(src.hue(), 36.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::Rainbow | Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() + 60.0),
                24.0,
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    src,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0],
                ),
                32.0,
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    src,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0],
                ),
                32.0,
            ),
            v => panic!("{v:?} variant not supported in Spec2021"),
        }
    }

    fn get_neutral_palette(
        &self,
        variant: Variant,
        src: &Hct,
        _: bool,
        _: Platform,
        _: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => {
                TonalPalette::from_hue_and_chroma(src.hue(), src.chroma() / 8.0)
            }
            Variant::FruitSalad | Variant::Vibrant => {
                TonalPalette::from_hue_and_chroma(src.hue(), 10.0)
            }
            Variant::Monochrome | Variant::Rainbow => {
                TonalPalette::from_hue_and_chroma(src.hue(), 0.0)
            }
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 2.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 6.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() + 15.0),
                8.0,
            ),
            v => panic!("{v:?} variant not supported in Spec2021"),
        }
    }

    fn get_neutral_variant_palette(
        &self,
        variant: Variant,
        src: &Hct,
        _: bool,
        _: Platform,
        _: f64,
    ) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => {
                TonalPalette::from_hue_and_chroma(src.hue(), src.chroma() / 8.0 + 4.0)
            }
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::Monochrome | Variant::Rainbow => {
                TonalPalette::from_hue_and_chroma(src.hue(), 0.0)
            }
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 2.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 8.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                MathUtils::sanitize_degrees_double(src.hue() + 15.0),
                12.0,
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(src.hue(), 12.0),
            v => panic!("{v:?} variant not supported in Spec2021"),
        }
    }

    fn get_error_palette(&self, _: Variant, _: &Hct, _: bool, _: Platform, _: f64) -> TonalPalette {
        TonalPalette::from_hue_and_chroma(25.0, 84.0)
    }
}
