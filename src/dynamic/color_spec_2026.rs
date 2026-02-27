/*
 * Copyright 2026 Google LLC
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

use crate::dynamic::color_spec::{ColorSpec, Platform, SpecVersion};
use crate::dynamic::color_spec_2025::ColorSpec2025;
use crate::dynamic::color_specs::ColorSpecs;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

pub struct ColorSpec2026 {
    base: ColorSpec2025,
    override_spec: SpecVersion,
}

impl Default for ColorSpec2026 {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpec2026 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: ColorSpec2025::with_override_spec(SpecVersion::Spec2026),
            override_spec: SpecVersion::Spec2026,
        }
    }

    // --- Internal Helpers ---

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

    fn get_contrast_curve(default_contrast: f64) -> ContrastCurve {
        match default_contrast {
            c if (c - 1.5).abs() < 1e-5 => ContrastCurve::new(1.5, 1.5, 3.0, 5.5),
            c if (c - 3.0).abs() < 1e-5 => ContrastCurve::new(3.0, 3.0, 4.5, 7.0),
            c if (c - 4.5).abs() < 1e-5 => ContrastCurve::new(4.5, 4.5, 7.0, 11.0),
            c if (c - 6.0).abs() < 1e-5 => ContrastCurve::new(6.0, 6.0, 7.0, 11.0),
            c if (c - 7.0).abs() < 1e-5 => ContrastCurve::new(7.0, 7.0, 11.0, 21.0),
            c if (c - 9.0).abs() < 1e-5 => ContrastCurve::new(9.0, 9.0, 11.0, 21.0),
            c if (c - 11.0).abs() < 1e-5 => ContrastCurve::new(11.0, 11.0, 21.0, 21.0),
            c if (c - 21.0).abs() < 1e-5 => ContrastCurve::new(21.0, 21.0, 21.0, 21.0),
            _ => ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0),
        }
    }
}

impl ColorSpec for ColorSpec2026 {
    // Inherit standard Palette Key Colors from Base
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

    // Inherit inherited surface roles from base (remaps)
    fn background(&self) -> Arc<DynamicColor> {
        self.base.background()
    }
    fn on_background(&self) -> Arc<DynamicColor> {
        self.base.on_background()
    }

    fn surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 4.0 } else { 98.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_dim".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 1.0 } else { 1.7 }
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 4.0 } else { 87.0 }
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
            .surface_dim()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_bright".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 1.7 } else { 1.0 }
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 18.0 } else { 98.0 }
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
            .surface_bright()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_lowest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 0.0 } else { 100.0 }
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
            .surface_container_lowest()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_low".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.25 } else { 0.0 },
            )),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 6.0 } else { 96.0 }
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
            .surface_container_low()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.4 } else { 0.0 },
            )),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 9.0 } else { 94.0 }
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
            .surface_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_high".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.5 } else { 0.0 },
            )),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 12.0 } else { 92.0 }
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
            .surface_container_high()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_highest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 },
            )),
            None,
            Some(Arc::new(|s| {
                if s.variant == Variant::Cmf {
                    if s.is_dark { 15.0 } else { 90.0 }
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
            .surface_container_highest()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 1.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.is_dark { 11.0 } else { 9.0 }))
            })),
            None,
            None,
        );
        self.base
            .on_surface()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        self.base.surface_variant()
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_surface_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 1.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|s| {
                Some(Self::get_contrast_curve(if s.is_dark { 6.0 } else { 4.5 }))
            })),
            None,
            None,
        );
        self.base
            .on_surface_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "inverse_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 1.0 },
            )),
            None,
            Some(Arc::new(|s| if s.is_dark { 98.0 } else { 4.0 })),
            None,
            None,
            None,
            None,
        );
        self.base
            .inverse_surface()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "outline".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 1.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(3.0)))),
            None,
            None,
        );
        self.base
            .outline()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "outline_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(
                |s| if s.variant == Variant::Cmf { 1.7 } else { 1.0 },
            )),
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(1.5)))),
            None,
            None,
        );
        self.base
            .outline_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn shadow(&self) -> Arc<DynamicColor> {
        self.base.shadow()
    }
    fn scrim(&self) -> Arc<DynamicColor> {
        self.base.scrim()
    }
    fn surface_tint(&self) -> Arc<DynamicColor> {
        self.base.surface_tint()
    }

    // --- Primaries ---

    fn primary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.source_color_hct().chroma() <= 12.0 {
                    if s.is_dark { 80.0 } else { 40.0 }
                } else {
                    s.source_color_hct().tone()
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .primary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.primary_dim()
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).primary())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_primary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let override_spec = [self.override_spec; 1];
        let color2026 = DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |s| {
                Some(ColorSpecs::get(override_spec[0]).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if !s.is_dark && s.source_color_hct().chroma() <= 12.0 {
                    90.0
                } else if s.source_color_hct().tone() > 55.0 {
                    s.source_color_hct().tone().clamp(61.0, 90.0)
                } else {
                    s.source_color_hct().tone().clamp(30.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(override_spec[0]).primary_container(),
                    ColorSpecs::get(override_spec[0]).primary(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .primary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let override_spec = [self.override_spec; 1];
        let color2026 = DynamicColor::new(
            "on_primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_s| {
                Some(ColorSpecs::get(override_spec[0]).primary_container())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_primary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        self.base.inverse_primary()
    }

    // --- Fixed Colors ---

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .primary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "primary_fixed_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).primary_fixed().get_tone(s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).primary_fixed_dim(),
                    ColorSpecs::get(s.spec_version).primary_fixed(),
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // --- Secondaries ---

    fn secondary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.is_dark {
                    Self::t_min_c(&s.secondary_palette, 0.0, 100.0)
                } else {
                    Self::t_max_c(&s.secondary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .secondary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.secondary_dim()
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_secondary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.is_dark {
                    Self::t_min_c(&s.secondary_palette, 20.0, 49.0)
                } else {
                    Self::t_max_c(&s.secondary_palette, 61.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).secondary_container(),
                    ColorSpecs::get(s.spec_version).secondary(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .secondary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).secondary_container())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_secondary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .secondary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary_fixed_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version)
                    .secondary_fixed()
                    .get_tone(s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).secondary_fixed_dim(),
                    ColorSpecs::get(s.spec_version).secondary_fixed(),
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // --- Tertiaries ---

    fn tertiary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                s.source_color_hct_list
                    .get(1)
                    .map_or_else(|| s.source_color_hct().tone(), Hct::tone)
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .tertiary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                let sec_hct = s
                    .source_color_hct_list
                    .get(1)
                    .unwrap_or_else(|| s.source_color_hct());
                if sec_hct.tone() > 55.0 {
                    sec_hct.tone().clamp(61.0, 90.0)
                } else {
                    sec_hct.tone().clamp(20.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).tertiary_container(),
                    ColorSpecs::get(s.spec_version).tertiary(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .tertiary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).tertiary_container())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                let temp_s = DynamicScheme::from_scheme_with_contrast(s, false, 0.0);
                ColorSpecs::get(s.spec_version)
                    .tertiary_container()
                    .get_tone(&temp_s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary_fixed_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                ColorSpecs::get(s.spec_version).tertiary_fixed().get_tone(s)
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).tertiary_fixed_dim(),
                    ColorSpecs::get(s.spec_version).tertiary_fixed(),
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
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
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // --- Errors ---

    fn error(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                Self::t_max_c(&s.error_palette, 0.0, 100.0, 1.0)
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .error()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| Some(ColorSpecs::get(s.spec_version).error()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_error()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).highest_surface(s))
            })),
            Some(Arc::new(|s| {
                if s.is_dark {
                    Self::t_min_c(&s.error_palette, 0.0, 100.0)
                } else {
                    Self::t_max_c(&s.error_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|s| {
                if s.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(|s| {
                Some(ToneDeltaPair::new(
                    ColorSpecs::get(s.spec_version).error_container(),
                    ColorSpecs::get(s.spec_version).error(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    true,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .error_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(|s| {
                Some(ColorSpecs::get(s.spec_version).error_container())
            })),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_error_container()
            .extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // Pass-through standard base logic for HCT and Tone calculations
    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        self.base.get_hct(scheme, color)
    }
    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        self.base.get_tone(scheme, color)
    }

    // Palette getters remain the same as base (2025)
    fn get_primary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base.get_primary_palette(variant, src, dark, plat, cl)
    }
    fn get_secondary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base
            .get_secondary_palette(variant, src, dark, plat, cl)
    }
    fn get_tertiary_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base.get_tertiary_palette(variant, src, dark, plat, cl)
    }
    fn get_neutral_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base.get_neutral_palette(variant, src, dark, plat, cl)
    }
    fn get_neutral_variant_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base
            .get_neutral_variant_palette(variant, src, dark, plat, cl)
    }
    fn get_error_palette(
        &self,
        variant: Variant,
        src: &Hct,
        dark: bool,
        plat: Platform,
        cl: f64,
    ) -> TonalPalette {
        self.base.get_error_palette(variant, src, dark, plat, cl)
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.tertiary_dim()
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.error_dim()
    }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        let spec = ColorSpecs::get(scheme.spec_version);
        if scheme.is_dark {
            spec.surface_bright()
        } else {
            spec.surface_dim()
        }
    }
}
