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
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::{DynamicColor, DynamicColorFunction};
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

/// [`ColorSpec`] implementation for the 2026 Material Design color specification.
pub struct ColorSpec2026 {
    base: ColorSpec2025,
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
            base: ColorSpec2025::new(),
        }
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Helper Methods
    // ────────────────────────────────────────────────────────────────────────────

    fn t_max_c(palette: &TonalPalette, lower_bound: f64, upper_bound: f64, chroma_multiplier: f64) -> f64 {
        let answer = Self::find_best_tone_for_chroma(palette.hue, palette.chroma * chroma_multiplier, 100.0, true);
        answer.clamp(lower_bound, upper_bound)
    }

    fn t_max_c_bounds(palette: &TonalPalette, lower_bound: f64, upper_bound: f64) -> f64 {
        Self::t_max_c(palette, lower_bound, upper_bound, 1.0)
    }

    fn t_max_c_default(palette: &TonalPalette) -> f64 {
        Self::t_max_c(palette, 0.0, 100.0, 1.0)
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

    fn get_highest_surface_bg(&self) -> DynamicColorFunction<Option<Arc<DynamicColor>>> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(move |s| {
            Some(if s.is_dark { surface_bright.clone() } else { surface_dim.clone() })
        })
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Static Tone Evaluators (For breaking cycles in ToneDeltaPairs)
    // ────────────────────────────────────────────────────────────────────────────

    fn primary_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.source_color_hct().chroma() <= 12.0 {
            if scheme.is_dark { 80.0 } else { 40.0 }
        } else {
            scheme.source_color_hct().tone()
        }
    }

    fn primary_container_tone(scheme: &DynamicScheme) -> f64 {
        if !scheme.is_dark && scheme.source_color_hct().chroma() <= 12.0 {
            90.0
        } else if scheme.source_color_hct().tone() > 55.0 {
            scheme.source_color_hct().tone().clamp(61.0, 90.0)
        } else {
            scheme.source_color_hct().tone().clamp(30.0, 49.0)
        }
    }

    fn primary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::primary_container_tone(&temp_s)
    }

    fn primary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::primary_fixed_tone(scheme)
    }

    fn secondary_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.is_dark {
            Self::t_min_c_default(&scheme.secondary_palette)
        } else {
            Self::t_max_c_default(&scheme.secondary_palette)
        }
    }

    fn secondary_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.is_dark {
            Self::t_min_c(&scheme.secondary_palette, 20.0, 49.0)
        } else {
            Self::t_max_c_bounds(&scheme.secondary_palette, 61.0, 90.0)
        }
    }

    fn secondary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::secondary_container_tone(&temp_s)
    }

    fn secondary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::secondary_fixed_tone(scheme)
    }

    fn tertiary_tone(scheme: &DynamicScheme) -> f64 {
        scheme.source_color_hct_list.get(1).map(|h| h.tone()).unwrap_or_else(|| scheme.source_color_hct().tone())
    }

    fn tertiary_container_tone(scheme: &DynamicScheme) -> f64 {
        let secondary_source_color_hct = scheme.source_color_hct_list.get(1).unwrap_or(scheme.source_color_hct());
        if secondary_source_color_hct.tone() > 55.0 {
            secondary_source_color_hct.tone().clamp(61.0, 90.0)
        } else {
            secondary_source_color_hct.tone().clamp(20.0, 49.0)
        }
    }

    fn tertiary_fixed_tone(scheme: &DynamicScheme) -> f64 {
        let temp_s = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
        Self::tertiary_container_tone(&temp_s)
    }

    fn tertiary_fixed_dim_tone(scheme: &DynamicScheme) -> f64 {
        Self::tertiary_fixed_tone(scheme)
    }

    fn error_tone(scheme: &DynamicScheme) -> f64 {
        Self::t_max_c_default(&scheme.error_palette)
    }

    fn error_container_tone(scheme: &DynamicScheme) -> f64 {
        if scheme.is_dark {
            Self::t_min_c_default(&scheme.error_palette)
        } else {
            Self::t_max_c_default(&scheme.error_palette)
        }
    }
}

impl ColorSpec for ColorSpec2026 {

    // ────────────────────────────────────────────────────────────────────────────
    // Un-overridden / Inherited Properties
    // ────────────────────────────────────────────────────────────────────────────

    fn primary_palette_key_color(&self) -> Arc<DynamicColor> { self.base.primary_palette_key_color() }
    fn secondary_palette_key_color(&self) -> Arc<DynamicColor> { self.base.secondary_palette_key_color() }
    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> { self.base.tertiary_palette_key_color() }
    fn neutral_palette_key_color(&self) -> Arc<DynamicColor> { self.base.neutral_palette_key_color() }
    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> { self.base.neutral_variant_palette_key_color() }
    fn error_palette_key_color(&self) -> Arc<DynamicColor> { self.base.error_palette_key_color() }
    fn background(&self) -> Arc<DynamicColor> { self.base.background() }
    fn on_background(&self) -> Arc<DynamicColor> { self.base.on_background() }
    fn surface_variant(&self) -> Arc<DynamicColor> { self.base.surface_variant() }
    fn shadow(&self) -> Arc<DynamicColor> { self.base.shadow() }
    fn scrim(&self) -> Arc<DynamicColor> { self.base.scrim() }
    fn surface_tint(&self) -> Arc<DynamicColor> { self.base.surface_tint() }
    fn primary_dim(&self) -> Option<Arc<DynamicColor>> { self.base.primary_dim() }
    fn inverse_primary(&self) -> Arc<DynamicColor> { self.base.inverse_primary() }
    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> { self.base.secondary_dim() }
    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> { self.base.tertiary_dim() }
    fn error_dim(&self) -> Option<Arc<DynamicColor>> { self.base.error_dim() }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        if scheme.is_dark {
            self.surface_bright()
        } else {
            self.surface_dim()
        }
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Surfaces [S]
    // ────────────────────────────────────────────────────────────────────────────

    fn surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 4.0 } else { 98.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_dim".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 1.0 } else { 1.7 } } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 4.0 } else { 87.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_dim().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_bright".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 1.7 } else { 1.0 } } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 18.0 } else { 98.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_bright().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_lowest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 0.0 } else { 100.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_container_lowest().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_low".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.25 } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 6.0 } else { 96.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_container_low().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.4 } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 9.0 } else { 94.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_high".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.5 } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 12.0 } else { 92.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_container_high().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "surface_container_highest".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { if s.is_dark { 15.0 } else { 90.0 } } else { 0.0 })),
            None, None, None, None,
        );
        self.base.surface_container_highest().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            Some(self.get_highest_surface_bg()),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.is_dark { 11.0 } else { 9.0 })))),
            None, None,
        );
        self.base.on_surface().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "on_surface_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            Some(self.get_highest_surface_bg()),
            None, None,
            Some(Arc::new(|s| Some(Self::get_contrast_curve(if s.is_dark { 6.0 } else { 4.5 })))),
            None, None,
        );
        self.base.on_surface_variant().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "outline".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            Some(self.get_highest_surface_bg()),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(3.0)))),
            None, None,
        );
        self.base.outline().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "outline_variant".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            Some(self.get_highest_surface_bg()),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(1.5)))),
            None, None,
        );
        self.base.outline_variant().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "inverse_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|s| if s.variant == Variant::Cmf { 1.7 } else { 0.0 })),
            None,
            Some(Arc::new(|s| if s.is_dark { 98.0 } else { 4.0 })),
            None, None, None, None,
        );
        self.base.inverse_surface().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let inv_surface = self.inverse_surface();
        let color2026 = DynamicColor::new(
            "inverse_on_surface".to_string(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(inv_surface.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.inverse_on_surface().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Primaries [P]
    // ────────────────────────────────────────────────────────────────────────────

    fn primary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::primary_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.primary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let color2026 = DynamicColor::new(
            "on_primary".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(primary.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_primary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
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

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                pc_stub.clone(),
                p_stub.clone(),
                5.0,
                TonePolarity::RelativeLighter,
                true,
                DeltaConstraint::Farther,
            ))
        });

        let color2026 = DynamicColor::new(
            "primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::primary_container_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.primary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let pc = self.primary_container();
        let color2026 = DynamicColor::new(
            "on_primary_container".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pc.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_primary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::primary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.primary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
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

        let color2026 = DynamicColor::new(
            "primary_fixed_dim".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::primary_fixed_dim_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.primary_fixed_dim().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_primary_fixed".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_primary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_primary_fixed_variant".to_string(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_primary_fixed_variant().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Secondaries [Q]
    // ────────────────────────────────────────────────────────────────────────────

    fn secondary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::secondary_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.secondary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let secondary = self.secondary();
        let color2026 = DynamicColor::new(
            "on_secondary".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(secondary.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_secondary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
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

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                sc_stub.clone(),
                s_stub.clone(),
                5.0,
                TonePolarity::RelativeLighter,
                true,
                DeltaConstraint::Farther,
            ))
        });

        let color2026 = DynamicColor::new(
            "secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::secondary_container_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.secondary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let sc = self.secondary_container();
        let color2026 = DynamicColor::new(
            "on_secondary_container".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sc.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_secondary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::secondary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.secondary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
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

        let color2026 = DynamicColor::new(
            "secondary_fixed_dim".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::secondary_fixed_dim_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.secondary_fixed_dim().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_secondary_fixed".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_secondary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_secondary_fixed_variant".to_string(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_secondary_fixed_variant().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Tertiaries [T]
    // ────────────────────────────────────────────────────────────────────────────

    fn tertiary(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::tertiary_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.tertiary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let tertiary = self.tertiary();
        let color2026 = DynamicColor::new(
            "on_tertiary".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tertiary.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_tertiary().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
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

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                tc_stub.clone(),
                t_stub.clone(),
                5.0,
                TonePolarity::RelativeLighter,
                true,
                DeltaConstraint::Farther,
            ))
        });

        let color2026 = DynamicColor::new(
            "tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::tertiary_container_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.tertiary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let tc = self.tertiary_container();
        let color2026 = DynamicColor::new(
            "on_tertiary_container".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tc.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_tertiary_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::tertiary_fixed_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            None, None,
        );
        self.base.tertiary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
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

        let color2026 = DynamicColor::new(
            "tertiary_fixed_dim".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::tertiary_fixed_dim_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.tertiary_fixed_dim().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_tertiary_fixed".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None, None,
        );
        self.base.on_tertiary_fixed().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim();
        let color2026 = DynamicColor::new(
            "on_tertiary_fixed_variant".to_string(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.on_tertiary_fixed_variant().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Errors [E]
    // ────────────────────────────────────────────────────────────────────────────

    fn error(&self) -> Arc<DynamicColor> {
        let color2026 = DynamicColor::new(
            "error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::error_tone)),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None, None,
        );
        self.base.error().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let error = self.error();
        let color2026 = DynamicColor::new(
            "on_error".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(error.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_error().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn error_container(&self) -> Arc<DynamicColor> {
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

        let tdp: DynamicColorFunction<Option<ToneDeltaPair>> = Arc::new(move |_| {
            Some(ToneDeltaPair::new(
                ec_stub.clone(),
                e_stub.clone(),
                5.0,
                TonePolarity::RelativeLighter,
                true,
                DeltaConstraint::Farther,
            ))
        });

        let color2026 = DynamicColor::new(
            "error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            true, None,
            Some(self.get_highest_surface_bg()),
            Some(Arc::new(Self::error_container_tone)),
            None,
            Some(Arc::new(|s| if s.contrast_level > 0.0 { Some(Self::get_contrast_curve(1.5)) } else { None })),
            Some(tdp),
            None,
        );
        self.base.error_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let ec = self.error_container();
        let color2026 = DynamicColor::new(
            "on_error_container".to_string(),
            Arc::new(|s| s.error_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(ec.clone()))),
            None, None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None, None,
        );
        self.base.on_error_container().extend_spec_version(SpecVersion::Spec2026, &color2026)
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Color value calculations and Scheme Palettes
    // ────────────────────────────────────────────────────────────────────────────
    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct { self.base.get_hct(scheme, color) }
    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 { self.base.get_tone(scheme, color) }

    fn get_primary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_primary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
    fn get_secondary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_secondary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
    fn get_tertiary_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_tertiary_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
    fn get_neutral_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_neutral_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
    fn get_neutral_variant_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_neutral_variant_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
    fn get_error_palette(&self, variant: Variant, source_color_hct: &Hct, is_dark: bool, platform: Platform, contrast_level: f64) -> TonalPalette {
        self.base.get_error_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
}