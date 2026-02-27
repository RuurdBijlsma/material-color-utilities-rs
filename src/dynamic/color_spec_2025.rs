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
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

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

    fn t_max_c(
        palette: &TonalPalette,
        lower_bound: f64,
        upper_bound: f64,
        chroma_multiplier: f64,
    ) -> f64 {
        let answer = Self::find_best_tone_for_chroma(
            palette.hue,
            palette.chroma * chroma_multiplier,
            100.0,
            true,
        );
        answer.clamp(lower_bound, upper_bound)
    }

    fn t_min_c(palette: &TonalPalette, lower_bound: f64, upper_bound: f64) -> f64 {
        let answer = Self::find_best_tone_for_chroma(palette.hue, palette.chroma, 0.0, false);
        answer.clamp(lower_bound, upper_bound)
    }

    fn find_best_tone_for_chroma(
        hue: f64,
        chroma: f64,
        mut tone: f64,
        by_decreasing_tone: bool,
    ) -> f64 {
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
        match default_contrast {
            c if (c - 1.5).abs() < f64::EPSILON => ContrastCurve::new(1.5, 1.5, 3.0, 5.5),
            c if (c - 3.0).abs() < f64::EPSILON => ContrastCurve::new(3.0, 3.0, 4.5, 7.0),
            c if (c - 4.5).abs() < f64::EPSILON => ContrastCurve::new(4.5, 4.5, 7.0, 11.0),
            c if (c - 6.0).abs() < f64::EPSILON => ContrastCurve::new(6.0, 6.0, 7.0, 11.0),
            c if (c - 7.0).abs() < f64::EPSILON => ContrastCurve::new(7.0, 7.0, 11.0, 21.0),
            c if (c - 9.0).abs() < f64::EPSILON => ContrastCurve::new(9.0, 9.0, 11.0, 21.0),
            c if (c - 11.0).abs() < f64::EPSILON => ContrastCurve::new(11.0, 11.0, 21.0, 21.0),
            c if (c - 21.0).abs() < f64::EPSILON => ContrastCurve::new(21.0, 21.0, 21.0, 21.0),
            _ => ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0),
        }
    }
}

impl ColorSpec for ColorSpec2025 {
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
        let spec = ColorSpecs::get(scheme.spec_version);
        if scheme.is_dark {
            spec.surface_bright()
        } else {
            spec.surface_dim()
        }
    }

    fn surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
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
            None,
            None,
            None,
            None,
        );
        self.base
            .surface()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn background(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "background".to_string(),
            Arc::new(|s| ColorSpecs::get(s.spec_version).surface().palette.clone()(s)),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).surface().get_tone(s)
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .background()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_background".to_string(),
            Arc::new(|s| ColorSpecs::get(s.spec_version).on_surface().palette.clone()(s)),
            false,
            None,
            Some(Arc::new(|s| {
                Some(
                    ColorSpecs::get(s.spec_version)
                        .on_surface()
                        .background
                        .as_ref()
                        .and_then(|f| f(s))?,
                )
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    100.0
                } else {
                    ColorSpecs::get(s.spec_version).on_surface().get_tone(s)
                }
            })),
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .on_surface()
                    .contrast_curve
                    .as_ref()
                    .and_then(|f| f(s))
            })),
            None,
            None,
        );
        self.base
            .on_background()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
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
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                2.7
                            } else {
                                1.75
                            }
                        }
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
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_dim()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
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
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                2.7
                            } else {
                                1.75
                            }
                        }
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
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_bright()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_lowest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 0.0 } else { 100.0 })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_container_lowest()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
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
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                1.3
                            } else {
                                1.15
                            }
                        }
                        Variant::Vibrant => 1.08,
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark {
                        6.0
                    } else if Hct::is_yellow(s.neutral_palette.hue) {
                        98.0
                    } else if s.variant == Variant::Vibrant {
                        95.0
                    } else {
                        96.0
                    }
                } else {
                    15.0
                }
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_container_low()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
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
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                1.6
                            } else {
                                1.3
                            }
                        }
                        Variant::Vibrant => 1.15,
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark {
                        9.0
                    } else if Hct::is_yellow(s.neutral_palette.hue) {
                        96.0
                    } else if s.variant == Variant::Vibrant {
                        92.0
                    } else {
                        94.0
                    }
                } else {
                    20.0
                }
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
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
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                1.95
                            } else {
                                1.45
                            }
                        }
                        Variant::Vibrant => 1.22,
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark {
                        12.0
                    } else if Hct::is_yellow(s.neutral_palette.hue) {
                        94.0
                    } else if s.variant == Variant::Vibrant {
                        90.0
                    } else {
                        92.0
                    }
                } else {
                    25.0
                }
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_container_high()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_container_highest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| match s.variant {
                Variant::Neutral => 2.2,
                Variant::TonalSpot => 1.7,
                Variant::Expressive => {
                    if Hct::is_yellow(s.neutral_palette.hue) {
                        2.3
                    } else {
                        1.6
                    }
                }
                Variant::Vibrant => 1.29,
                _ => 1.0,
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
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_container_highest()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                if s.is_dark { 3.0 } else { 2.3 }
                            } else {
                                1.6
                            }
                        }
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.variant == Variant::Vibrant {
                    Self::t_max_c(&s.neutral_palette, 0.0, 100.0, 1.1)
                } else {
                    ColorSpecs::get(s.spec_version)
                        .highest_surface(s)
                        .get_tone(s)
                }
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(
                    if s.is_dark && s.platform == Platform::Phone {
                        11.0
                    } else {
                        9.0
                    },
                ))
            })),
            None,
            None,
        );
        self.base
            .on_surface()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_variant".to_string(),
            Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .surface_container_highest()
                    .palette
                    .clone()(s)
            }),
            true,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .surface_container_highest()
                    .chroma_multiplier
                    .as_ref()
                    .map_or(1.0, |f| f(s))
            })),
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .surface_container_highest()
                    .get_tone(s)
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_surface_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                if s.is_dark { 3.0 } else { 2.3 }
                            } else {
                                1.6
                            }
                        }
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    if s.is_dark { 6.0 } else { 4.5 }
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_surface_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "inverse_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| if s.is_dark { 98.0 } else { 4.0 })),
            None,
            None,
            None,
            None,
        );
        self.base
            .inverse_surface()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "inverse_on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).inverse_surface())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .inverse_on_surface()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "outline".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                if s.is_dark { 3.0 } else { 2.3 }
                            } else {
                                1.6
                            }
                        }
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    3.0
                } else {
                    4.5
                }))
            })),
            None,
            None,
        );
        self.base
            .outline()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "outline_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    match s.variant {
                        Variant::Neutral => 2.2,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(s.neutral_palette.hue) {
                                if s.is_dark { 3.0 } else { 2.3 }
                            } else {
                                1.6
                            }
                        }
                        _ => 1.0,
                    }
                } else {
                    1.0
                }
            })),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    1.5
                } else {
                    3.0
                }))
            })),
            None,
            None,
        );
        self.base
            .outline_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn surface_tint(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "surface_tint".to_string(),
            Arc::new(|s| ColorSpecs::get(s.spec_version).primary().palette.clone()(s)),
            true,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .primary()
                    .chroma_multiplier
                    .as_ref()
                    .map_or(1.0, |f| f(s))
            })),
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).primary().get_tone(s)
            })),
            None,
            None,
            None,
            None,
        );
        self.base
            .surface_tint()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // --- Primaries ---

    fn primary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| match s.variant {
                Variant::Neutral => {
                    if s.platform == Platform::Phone {
                        if s.is_dark { 80.0 } else { 40.0 }
                    } else {
                        90.0
                    }
                }
                Variant::TonalSpot => {
                    if s.platform == Platform::Phone {
                        if s.is_dark {
                            80.0
                        } else {
                            Self::t_max_c(&s.primary_palette, 0.0, 100.0, 1.0)
                        }
                    } else {
                        Self::t_max_c(&s.primary_palette, 0.0, 90.0, 1.0)
                    }
                }
                Variant::Expressive => {
                    if s.platform == Platform::Phone {
                        let hue = s.primary_palette.hue;
                        let upper = if Hct::is_yellow(hue) {
                            25.0
                        } else if Hct::is_cyan(hue) {
                            88.0
                        } else {
                            98.0
                        };
                        Self::t_max_c(&s.primary_palette, 0.0, upper, 1.0)
                    } else {
                        Self::t_max_c(&s.primary_palette, 0.0, 100.0, 1.0)
                    }
                }
                _ => {
                    if s.platform == Platform::Phone {
                        let upper = if Hct::is_cyan(s.primary_palette.hue) {
                            88.0
                        } else {
                            98.0
                        };
                        Self::t_max_c(&s.primary_palette, 0.0, upper, 1.0)
                    } else {
                        Self::t_max_c(&s.primary_palette, 0.0, 100.0, 1.0)
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    4.5
                } else {
                    7.0
                }))
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(ToneDeltaPair::new(
                        spec.primary_container(),
                        spec.primary(),
                        5.0,
                        TonePolarity::RelativeLighter,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .primary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        Some(Arc::new(DynamicColor::new(
            "primary_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).surface_container_high())
            })),
            Some(Arc::new(|s| match s.variant {
                Variant::Neutral => 85.0,
                Variant::TonalSpot => Self::t_max_c(&s.primary_palette, 0.0, 90.0, 1.0),
                _ => Self::t_max_c(&s.primary_palette, 0.0, 100.0, 1.0),
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(Arc::new(|s| {
                // Workaround because we don't have inheritance:
                // Just set the spec here manually or else it gets the wrong spec
                let spec = ColorSpecs::get(SpecVersion::Spec2025);
                let spec_primary_dim = spec.primary_dim()?;
                 Some(ToneDeltaPair::new(
                    spec_primary_dim,
                    spec.primary(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        )))
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(spec.primary())
                } else {
                    spec.primary_dim()
                }
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_primary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    30.0
                } else {
                    match s.variant {
                        Variant::Neutral => {
                            if s.is_dark {
                                30.0
                            } else {
                                90.0
                            }
                        }
                        Variant::TonalSpot => {
                            if s.is_dark {
                                Self::t_min_c(&s.primary_palette, 35.0, 93.0)
                            } else {
                                Self::t_max_c(&s.primary_palette, 0.0, 90.0, 1.0)
                            }
                        }
                        Variant::Expressive => {
                            if s.is_dark {
                                Self::t_max_c(&s.primary_palette, 30.0, 93.0, 1.0)
                            } else {
                                let upper = if Hct::is_cyan(s.primary_palette.hue) {
                                    88.0
                                } else {
                                    90.0
                                };
                                Self::t_max_c(&s.primary_palette, 78.0, upper, 1.0)
                            }
                        }
                        _ => {
                            if s.is_dark {
                                Self::t_min_c(&s.primary_palette, 66.0, 93.0)
                            } else {
                                let upper = if Hct::is_cyan(s.primary_palette.hue) {
                                    88.0
                                } else {
                                    93.0
                                };
                                Self::t_max_c(&s.primary_palette, 66.0, upper, 1.0)
                            }
                        }
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Watch {
                    Some(ToneDeltaPair::new(
                        spec.primary_container(),
                        spec.primary_dim()?,
                        10.0,
                        TonePolarity::Darker,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .primary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_container())
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_primary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "inverse_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).inverse_surface())
            })),
            Some(Arc::new(|s| {
                Self::t_max_c(&s.primary_palette, 0.0, 100.0, 1.0)
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .inverse_primary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // --- Secondaries ---

    fn secondary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    if s.variant == Variant::Neutral {
                        90.0
                    } else {
                        Self::t_max_c(&s.secondary_palette, 0.0, 90.0, 1.0)
                    }
                } else {
                    match s.variant {
                        Variant::Neutral => {
                            if s.is_dark {
                                Self::t_min_c(&s.secondary_palette, 0.0, 98.0)
                            } else {
                                Self::t_max_c(&s.secondary_palette, 0.0, 100.0, 1.0)
                            }
                        }
                        Variant::Vibrant => Self::t_max_c(
                            &s.secondary_palette,
                            0.0,
                            if s.is_dark { 90.0 } else { 98.0 },
                            1.0,
                        ),
                        _ => {
                            if s.is_dark {
                                80.0
                            } else {
                                Self::t_max_c(&s.secondary_palette, 0.0, 100.0, 1.0)
                            }
                        }
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    4.5
                } else {
                    7.0
                }))
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    let spec = ColorSpecs::get(s.spec_version);
                    Some(ToneDeltaPair::new(
                        spec.secondary_container(),
                        spec.secondary(),
                        5.0,
                        TonePolarity::RelativeLighter,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .secondary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        Some(Arc::new(DynamicColor::new(
            "secondary_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).surface_container_high())
            })),
            Some(Arc::new(|s| {
                if s.variant == Variant::Neutral {
                    85.0
                } else {
                    Self::t_max_c(&s.secondary_palette, 0.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(Arc::new(|s| {
                // Workaround because we don't have inheritance:
                // Just set the spec here manually or else it gets the wrong spec
                let spec = ColorSpecs::get(SpecVersion::Spec2025);
                Some(ToneDeltaPair::new(
                    spec.secondary_dim()?,
                    spec.secondary(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        )))
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(spec.secondary())
                } else {
                    spec.secondary_dim()
                }
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_secondary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    30.0
                } else {
                    match s.variant {
                        Variant::Vibrant => {
                            if s.is_dark {
                                Self::t_min_c(&s.secondary_palette, 30.0, 40.0)
                            } else {
                                Self::t_max_c(&s.secondary_palette, 84.0, 90.0, 1.0)
                            }
                        }
                        Variant::Expressive => {
                            if s.is_dark {
                                15.0
                            } else {
                                Self::t_max_c(&s.secondary_palette, 90.0, 95.0, 1.0)
                            }
                        }
                        _ => {
                            if s.is_dark {
                                25.0
                            } else {
                                90.0
                            }
                        }
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Watch {
                    Some(ToneDeltaPair::new(
                        spec.secondary_container(),
                        spec.secondary_dim()?,
                        10.0,
                        TonePolarity::Darker,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .secondary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_container())
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_secondary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // --- Tertiaries ---

    fn tertiary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    if s.variant == Variant::TonalSpot {
                        Self::t_max_c(&s.tertiary_palette, 0.0, 90.0, 1.0)
                    } else {
                        Self::t_max_c(&s.tertiary_palette, 0.0, 100.0, 1.0)
                    }
                } else {
                    match s.variant {
                        Variant::Expressive | Variant::Vibrant => {
                            let upper = if Hct::is_cyan(s.tertiary_palette.hue) {
                                88.0
                            } else if s.is_dark {
                                98.0
                            } else {
                                100.0
                            };
                            Self::t_max_c(&s.tertiary_palette, 0.0, upper, 1.0)
                        }
                        _ => {
                            if s.is_dark {
                                Self::t_max_c(&s.tertiary_palette, 0.0, 98.0, 1.0)
                            } else {
                                Self::t_max_c(&s.tertiary_palette, 0.0, 100.0, 1.0)
                            }
                        }
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    4.5
                } else {
                    7.0
                }))
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    let spec = ColorSpecs::get(s.spec_version);
                    Some(ToneDeltaPair::new(
                        spec.tertiary_container(),
                        spec.tertiary(),
                        5.0,
                        TonePolarity::RelativeLighter,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .tertiary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        Some(Arc::new(DynamicColor::new(
            "tertiary_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).surface_container_high())
            })),
            Some(Arc::new(|s| {
                if s.variant == Variant::TonalSpot {
                    Self::t_max_c(&s.tertiary_palette, 0.0, 90.0, 1.0)
                } else {
                    Self::t_max_c(&s.tertiary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(Arc::new(|s| {
                // Workaround because we don't have inheritance:
                // Just set the spec here manually or else it gets the wrong spec
                let spec = ColorSpecs::get(SpecVersion::Spec2025);
                Some(ToneDeltaPair::new(
                    spec.tertiary_dim()?,
                    spec.tertiary(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        )))
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(spec.tertiary())
                } else {
                    spec.tertiary_dim()
                }
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_tertiary()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    if s.variant == Variant::TonalSpot {
                        Self::t_max_c(&s.tertiary_palette, 0.0, 90.0, 1.0)
                    } else {
                        Self::t_max_c(&s.tertiary_palette, 0.0, 100.0, 1.0)
                    }
                } else {
                    match s.variant {
                        Variant::Neutral => {
                            if s.is_dark {
                                Self::t_max_c(&s.tertiary_palette, 0.0, 93.0, 1.0)
                            } else {
                                Self::t_max_c(&s.tertiary_palette, 0.0, 96.0, 1.0)
                            }
                        }
                        Variant::TonalSpot => Self::t_max_c(
                            &s.tertiary_palette,
                            0.0,
                            if s.is_dark { 93.0 } else { 100.0 },
                            1.0,
                        ),
                        Variant::Expressive => {
                            let upper = if Hct::is_cyan(s.tertiary_palette.hue) {
                                88.0
                            } else if s.is_dark {
                                93.0
                            } else {
                                100.0
                            };
                            Self::t_max_c(&s.tertiary_palette, 75.0, upper, 1.0)
                        }
                        _ => {
                            if s.is_dark {
                                Self::t_max_c(&s.tertiary_palette, 0.0, 93.0, 1.0)
                            } else {
                                Self::t_max_c(&s.tertiary_palette, 72.0, 100.0, 1.0)
                            }
                        }
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Watch {
                    Some(ToneDeltaPair::new(
                        spec.tertiary_container(),
                        spec.tertiary_dim()?,
                        10.0,
                        TonePolarity::Darker,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .tertiary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_container())
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_tertiary_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // --- Error Colors ---

    fn error(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    if s.is_dark {
                        Self::t_min_c(&s.error_palette, 0.0, 98.0)
                    } else {
                        Self::t_max_c(&s.error_palette, 0.0, 100.0, 1.0)
                    }
                } else {
                    Self::t_min_c(&s.error_palette, 0.0, 100.0)
                }
            })),
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    4.5
                } else {
                    7.0
                }))
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(ToneDeltaPair::new(
                        spec.error_container(),
                        spec.error(),
                        5.0,
                        TonePolarity::RelativeLighter,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .error()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        Some(Arc::new(DynamicColor::new(
            "error_dim".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).surface_container_high())
            })),
            Some(Arc::new(|s| Self::t_min_c(&s.error_palette, 0.0, 100.0))),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            Some(Arc::new(|s| {
                // Workaround because we don't have inheritance:
                // Just set the spec here manually or else it gets the wrong spec
                let spec = ColorSpecs::get(SpecVersion::Spec2025);
                Some(ToneDeltaPair::new(
                    spec.error_dim()?,
                    spec.error(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        )))
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Phone {
                    Some(spec.error())
                } else {
                    spec.error_dim()
                }
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    6.0
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_error()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                if s.platform == Platform::Watch {
                    30.0
                } else {
                    if s.is_dark {
                        Self::t_min_c(&s.error_palette, 30.0, 93.0)
                    } else {
                        Self::t_max_c(&s.error_palette, 0.0, 90.0, 1.0)
                    }
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                if s.platform == Platform::Watch {
                    Some(ToneDeltaPair::new(
                        spec.error_container(),
                        spec.error_dim()?,
                        10.0,
                        TonePolarity::Darker,
                        true,
                        DeltaConstraint::Farther,
                    ))
                } else {
                    None
                }
            })),
            None,
        );
        self.base
            .error_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).error_container())
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.platform == Platform::Phone {
                    4.5
                } else {
                    7.0
                }))
            })),
            None,
            None,
        );
        self.base
            .on_error_container()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    // --- Fixed Colors ---

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .primary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        );
        self.base
            .primary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "primary_fixed_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).primary_fixed().get_tone(s)
            })),
            None,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.primary_fixed_dim(),
                    spec.primary_fixed(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .primary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_primary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_primary_fixed_variant".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_primary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .secondary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        );
        self.base
            .secondary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "secondary_fixed_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .secondary_fixed()
                    .get_tone(s)
            })),
            None,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.secondary_fixed_dim(),
                    spec.secondary_fixed(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .secondary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_secondary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_secondary_fixed_variant".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_secondary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone {
                    Some(ColorSpecs::get(s.spec_version).highest_surface(s))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .tertiary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.platform == Platform::Phone && s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        );
        self.base
            .tertiary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "tertiary_fixed_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).tertiary_fixed().get_tone(s)
            })),
            None,
            None,
            Some(Arc::new(|s| {
                let spec = ColorSpecs::get(s.spec_version);
                Some(ToneDeltaPair::new(
                    spec.tertiary_fixed_dim(),
                    spec.tertiary_fixed(),
                    5.0,
                    TonePolarity::Darker,
                    true,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .tertiary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary_fixed()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2025 = DynamicColor::new(
            "on_tertiary_fixed_variant".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_fixed_dim())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_tertiary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2025, &color2025)
    }

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        let palette = (color.palette)(scheme);
        let tone = self.get_tone(scheme, color);
        let chroma_multiplier = color.chroma_multiplier.as_ref().map_or(1.0, |f| f(scheme));
        Hct::from(palette.hue, palette.chroma * chroma_multiplier, tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        let tone_delta_pair = color.tone_delta_pair.as_ref().and_then(|f| f(scheme));
        if let Some(tdp) = tone_delta_pair {
            let (role_a, role_b) = (&tdp.role_a, &tdp.role_b);
            let absolute_delta = if tdp.polarity == TonePolarity::Darker
                || (tdp.polarity == TonePolarity::RelativeLighter && scheme.is_dark)
                || (tdp.polarity == TonePolarity::RelativeDarker && !scheme.is_dark)
            {
                -tdp.delta
            } else {
                tdp.delta
            };

            let am_role_a = color.name == role_a.name;
            let reference_role = if am_role_a { role_b } else { role_a };

            let mut self_tone = (color.tone)(scheme);
            let reference_tone = reference_role.get_tone(scheme);
            let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };

            match tdp.constraint {
                DeltaConstraint::Exact => {
                    self_tone = (reference_tone + relative_delta).clamp(0.0, 100.0)
                }
                DeltaConstraint::Nearer => {
                    if relative_delta > 0.0 {
                        self_tone = self_tone
                            .clamp(
                                reference_tone,
                                (reference_tone + relative_delta).max(reference_tone),
                            )
                            .clamp(0.0, 100.0);
                    } else {
                        self_tone = self_tone
                            .clamp(
                                (reference_tone + relative_delta).min(reference_tone),
                                reference_tone,
                            )
                            .clamp(0.0, 100.0);
                    }
                }
                DeltaConstraint::Farther => {
                    if relative_delta > 0.0 {
                        self_tone =
                            self_tone.clamp((reference_tone + relative_delta).min(100.0), 100.0);
                    } else {
                        self_tone =
                            self_tone.clamp(0.0, (reference_tone + relative_delta).max(0.0));
                    }
                }
            }

            if let (Some(bg_fn), Some(cc_fn)) =
                (color.background.as_ref(), color.contrast_curve.as_ref())
            {
                if let (Some(bg), Some(cc)) = (bg_fn(scheme), cc_fn(scheme)) {
                    let bg_tone = bg.get_tone(scheme);
                    let self_contrast = cc.get(scheme.contrast_level);
                    if !(Contrast::ratio_of_tones(bg_tone, self_tone) >= self_contrast
                        && scheme.contrast_level >= 0.0)
                    {
                        self_tone = DynamicColor::foreground_tone(bg_tone, self_contrast);
                    }
                }
            }
            if color.is_background && !color.name.ends_with("_fixed_dim") {
                self_tone = if self_tone >= 57.0 {
                    self_tone.clamp(65.0, 100.0)
                } else {
                    self_tone.clamp(0.0, 49.0)
                };
            }
            self_tone
        } else {
            let mut answer = (color.tone)(scheme);
            if let (Some(bg_fn), Some(cc_fn)) =
                (color.background.as_ref(), color.contrast_curve.as_ref())
            {
                if let (Some(bg), Some(cc)) = (bg_fn(scheme), cc_fn(scheme)) {
                    let bg_tone = bg.get_tone(scheme);
                    let desired_ratio = cc.get(scheme.contrast_level);
                    if !(Contrast::ratio_of_tones(bg_tone, answer) >= desired_ratio
                        && scheme.contrast_level >= 0.0)
                    {
                        answer = DynamicColor::foreground_tone(bg_tone, desired_ratio);
                    }
                }
            }
            if color.is_background && !color.name.ends_with("_fixed_dim") {
                answer = if answer >= 57.0 {
                    answer.clamp(65.0, 100.0)
                } else {
                    answer.clamp(0.0, 49.0)
                };
            }
            if let Some(bg2_fn) = color.second_background.as_ref() {
                if let (Some(bg1), Some(bg2), Some(cc_fn)) = (
                    color.background.as_ref().and_then(|f| f(scheme)),
                    bg2_fn(scheme),
                    color.contrast_curve.as_ref().and_then(|f| f(scheme)),
                ) {
                    let (t1, t2) = (bg1.get_tone(scheme), bg2.get_tone(scheme));
                    let desired = cc_fn.get(scheme.contrast_level);
                    if Contrast::ratio_of_tones(t1.max(t2), answer) < desired
                        || Contrast::ratio_of_tones(t1.min(t2), answer) < desired
                    {
                        let light = Contrast::lighter(t1.max(t2), desired);
                        let dark = Contrast::darker(t1.min(t2), desired);
                        if DynamicColor::tone_prefers_light_foreground(t1)
                            || DynamicColor::tone_prefers_light_foreground(t2)
                        {
                            return light.unwrap_or(100.0);
                        }
                        return dark.or(light).unwrap_or(0.0);
                    }
                }
            }
            answer
        }
    }

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
                    if Hct::is_blue(source_color_hct.hue()) {
                        12.0
                    } else {
                        8.0
                    }
                } else {
                    if Hct::is_blue(source_color_hct.hue()) {
                        16.0
                    } else {
                        12.0
                    }
                },
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone && is_dark {
                    26.0
                } else {
                    32.0
                },
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    if is_dark { 36.0 } else { 48.0 }
                } else {
                    40.0
                },
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    74.0
                } else {
                    56.0
                },
            ),
            _ => self.base.get_primary_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
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
                    if Hct::is_blue(source_color_hct.hue()) {
                        6.0
                    } else {
                        4.0
                    }
                } else {
                    if Hct::is_blue(source_color_hct.hue()) {
                        10.0
                    } else {
                        6.0
                    }
                },
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(source_color_hct.hue(), 16.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-160.0, 155.0, -100.0, 96.0, -96.0, -156.0, -165.0, -160.0],
                ),
                if platform == Platform::Phone {
                    if is_dark { 16.0 } else { 24.0 }
                } else {
                    24.0
                },
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                    &[-14.0, 10.0, -14.0, 10.0, -14.0],
                ),
                if platform == Platform::Phone {
                    56.0
                } else {
                    36.0
                },
            ),
            _ => self.base.get_secondary_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
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
                if platform == Platform::Phone {
                    20.0
                } else {
                    36.0
                },
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 20.0, 71.0, 161.0, 333.0, 360.0],
                    &[-40.0, 48.0, -32.0, 40.0, -32.0],
                ),
                if platform == Platform::Phone {
                    28.0
                } else {
                    32.0
                },
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0],
                    &[-165.0, 160.0, -105.0, 101.0, -101.0, -160.0, -170.0, -165.0],
                ),
                48.0,
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 71.0, 105.0, 140.0, 161.0, 253.0, 333.0, 360.0],
                    &[-72.0, 35.0, 24.0, -24.0, 62.0, 50.0, 62.0, -72.0],
                ),
                56.0,
            ),
            _ => self.base.get_tertiary_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
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
                if platform == Platform::Phone {
                    1.4
                } else {
                    6.0
                },
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                if platform == Platform::Phone {
                    5.0
                } else {
                    10.0
                },
            ),
            Variant::Expressive => {
                let h = DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
                    &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
                );
                let c = if platform == Platform::Phone {
                    if is_dark {
                        if Hct::is_yellow(h) { 6.0 } else { 14.0 }
                    } else {
                        18.0
                    }
                } else {
                    12.0
                };
                TonalPalette::from_hue_and_chroma(h, c)
            }
            Variant::Vibrant => {
                let h = DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                    &[-14.0, 10.0, -14.0, 10.0, -14.0],
                );
                let c = if platform == Platform::Phone {
                    28.0
                } else {
                    if Hct::is_blue(h) { 28.0 } else { 20.0 }
                };
                TonalPalette::from_hue_and_chroma(h, c)
            }
            _ => self.base.get_neutral_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
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
                (if platform == Platform::Phone {
                    1.4
                } else {
                    6.0
                }) * 2.2,
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                source_color_hct.hue(),
                (if platform == Platform::Phone {
                    5.0
                } else {
                    10.0
                }) * 1.7,
            ),
            Variant::Expressive => {
                let h = DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
                    &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
                );
                let c = if platform == Platform::Phone {
                    if is_dark {
                        if Hct::is_yellow(h) { 6.0 } else { 14.0 }
                    } else {
                        18.0
                    }
                } else {
                    12.0
                };
                TonalPalette::from_hue_and_chroma(
                    h,
                    c * if h >= 105.0 && h < 125.0 { 1.6 } else { 2.3 },
                )
            }
            Variant::Vibrant => {
                let h = DynamicScheme::get_rotated_hue(
                    source_color_hct,
                    &[0.0, 38.0, 105.0, 140.0, 333.0, 360.0],
                    &[-14.0, 10.0, -14.0, 10.0, -14.0],
                );
                let c = if platform == Platform::Phone {
                    28.0
                } else {
                    if Hct::is_blue(h) { 28.0 } else { 20.0 }
                };
                TonalPalette::from_hue_and_chroma(h, c * 1.29)
            }
            _ => self.base.get_neutral_variant_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
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
            Variant::Neutral => TonalPalette::from_hue_and_chroma(
                error_hue,
                if platform == Platform::Phone {
                    50.0
                } else {
                    40.0
                },
            ),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(
                error_hue,
                if platform == Platform::Phone {
                    60.0
                } else {
                    48.0
                },
            ),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                error_hue,
                if platform == Platform::Phone {
                    64.0
                } else {
                    48.0
                },
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                error_hue,
                if platform == Platform::Phone {
                    80.0
                } else {
                    60.0
                },
            ),
            _ => self.base.get_error_palette(
                variant,
                source_color_hct,
                is_dark,
                platform,
                contrast_level,
            ),
        }
    }
}
