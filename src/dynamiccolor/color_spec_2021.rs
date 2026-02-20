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

use crate::contrast::contrast::Contrast;
use crate::dislike::dislike_analyzer::DislikeAnalyzer;
use crate::dynamiccolor::color_spec::{ColorSpec, Platform};
use crate::dynamiccolor::contrast_curve::ContrastCurve;
use crate::dynamiccolor::dynamic_color::DynamicColor;
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;
use crate::dynamiccolor::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamiccolor::variant::Variant;
use crate::hct::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::temperature::temperature_cache::TemperatureCache;
use crate::utils::math_utils::MathUtils;

// ─── helpers ────────────────────────────────────────────────────────────────

fn is_fidelity(scheme: &DynamicScheme) -> bool {
    scheme.variant == Variant::Fidelity || scheme.variant == Variant::Content
}

fn is_monochrome(scheme: &DynamicScheme) -> bool {
    scheme.variant == Variant::Monochrome
}

/// Find a tone that best achieves `chroma` for the given `hue`.
fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing_tone: bool) -> f64 {
    let mut answer = tone;
    let mut closest = Hct::from(hue, chroma, tone);
    if closest.chroma() < chroma {
        let mut chroma_peak = closest.chroma();
        loop {
            answer += if by_decreasing_tone { -1.0 } else { 1.0 };
            let candidate = Hct::from(hue, chroma, answer);
            if chroma_peak > candidate.chroma() {
                break;
            }
            if (candidate.chroma() - chroma).abs() < 0.4 {
                break;
            }
            if (candidate.chroma() - chroma).abs() < (closest.chroma() - chroma).abs() {
                closest = candidate.clone();
            }
            chroma_peak = chroma_peak.max(candidate.chroma());
            if closest.chroma() >= chroma {
                break;
            }
        }
    }
    answer
}

// ─── ColorSpec2021 ──────────────────────────────────────────────────────────

/// [ColorSpec] implementation for the 2021 Material Design color specification.
pub struct ColorSpec2021;

impl ColorSpec2021 {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorSpec2021 {
    fn default() -> Self { Self::new() }
}

impl ColorSpec for ColorSpec2021 {
    // ── Key colors ────────────────────────────────────────────────────────

    fn primary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "primary_palette_key_color".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.primary_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    fn secondary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "secondary_palette_key_color".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.secondary_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    fn tertiary_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "tertiary_palette_key_color".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.tertiary_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    fn neutral_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "neutral_palette_key_color".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.neutral_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    fn neutral_variant_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "neutral_variant_palette_key_color".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.neutral_variant_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    fn error_palette_key_color(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "error_palette_key_color".into(),
            Arc::new(|s| s.error_palette.clone()),
            false, None, None,
            Some(Arc::new(|s| s.error_palette.key_color.tone())),
            None, None, None, None,
        ))
    }

    // ── Surfaces ─────────────────────────────────────────────────────────

    fn background(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "background".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 })),
            None, None, None, None,
        ))
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        let bg: Arc<DynamicColor> = self.background();
        Arc::new(DynamicColor::new(
            "on_background".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(bg.clone()))),
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 3.0, 4.5, 7.0)))),
            None, None,
        ))
    }

    fn surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.is_dark { 6.0 } else { 98.0 })),
            None, None, None, None,
        ))
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_dim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark { 6.0 } else { ContrastCurve::new(87.0, 87.0, 80.0, 75.0).get(s.contrast_level) }
            })),
            None, None, None, None,
        ))
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_bright".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark { ContrastCurve::new(24.0, 24.0, 29.0, 34.0).get(s.contrast_level) } else { 98.0 }
            })),
            None, None, None, None,
        ))
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_lowest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark { ContrastCurve::new(4.0, 4.0, 2.0, 0.0).get(s.contrast_level) } else { 100.0 }
            })),
            None, None, None, None,
        ))
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_low".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(10.0, 10.0, 11.0, 12.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(96.0, 96.0, 96.0, 95.0).get(s.contrast_level)
                }
            })),
            None, None, None, None,
        ))
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(12.0, 12.0, 16.0, 20.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(94.0, 94.0, 92.0, 90.0).get(s.contrast_level)
                }
            })),
            None, None, None, None,
        ))
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_high".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(17.0, 17.0, 21.0, 25.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(92.0, 92.0, 88.0, 85.0).get(s.contrast_level)
                }
            })),
            None, None, None, None,
        ))
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_highest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| {
                if s.is_dark {
                    ContrastCurve::new(22.0, 22.0, 26.0, 30.0).get(s.contrast_level)
                } else {
                    ContrastCurve::new(90.0, 90.0, 84.0, 80.0).get(s.contrast_level)
                }
            })),
            None, None, None, None,
        ))
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(move |s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 10.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 })),
            None, None, None, None,
        ))
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(move |s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "on_surface_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 30.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None, None,
        ))
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "inverse_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.is_dark { 90.0 } else { 20.0 })),
            None, None, None, None,
        ))
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let inv = self.inverse_surface();
        Arc::new(DynamicColor::new(
            "inverse_on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None,
            Some(Arc::new(move |_| Some(inv.clone()))),
            Some(Arc::new(|s| if s.is_dark { 20.0 } else { 95.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(move |s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "outline".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 60.0 } else { 50.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.5, 3.0, 4.5, 7.0)))),
            None, None,
        ))
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(move |s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "outline_variant".into(),
            Arc::new(|s| s.neutral_variant_palette.clone()),
            false, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 80.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None, None,
        ))
    }

    fn shadow(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "shadow".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None, None,
            Some(Arc::new(|_| 0.0)),
            None, None, None, None,
        ))
    }

    fn scrim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "scrim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false, None, None,
            Some(Arc::new(|_| 0.0)),
            None, None, None, None,
        ))
    }

    fn surface_tint(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_tint".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ))
    }

    // PRIMARIES, SECONDARIES, TERTIARIES, ERRORS, FIXED, highest_surface,
    // get_hct, get_tone & palette builders follow in subsequent sections.
    // They will be filled in via replace_file_content below.

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> { None }
    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> { None }
    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> { None }
    fn error_dim(&self) -> Option<Arc<DynamicColor>> { None }

    fn primary(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let primary_container = self.primary_container();
        let primary_self = Arc::new(DynamicColor::new(
            "primary".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 100.0 } else { 0.0 } }
                else { if s.is_dark { 80.0 } else { 40.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            None, None,
        ));
        // Build tone_delta_pair using Arc clones — rebuild without circular ref
        let pc = primary_container.clone();
        let ps = primary_self.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(pc.clone(), ps.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "primary".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 100.0 } else { 0.0 } }
                else { if s.is_dark { 80.0 } else { 40.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(tdp), None,
        ))
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        Arc::new(DynamicColor::new(
            "on_primary".into(), Arc::new(|s| s.primary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(primary.clone()))),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 10.0 } else { 90.0 } }
                else { if s.is_dark { 20.0 } else { 100.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let primary_stub = Arc::new(DynamicColor::new(
            "primary".into(), Arc::new(|s| s.primary_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let pc_self = Arc::new(DynamicColor::new(
            "primary_container".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| {
                if is_fidelity(s) { s.source_color_hct().tone() }
                else if is_monochrome(s) { if s.is_dark { 85.0 } else { 25.0 } }
                else { if s.is_dark { 30.0 } else { 90.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None, None,
        ));
        let pc2 = pc_self.clone();
        let ps2 = primary_stub.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(pc2.clone(), ps2.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "primary_container".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| {
                if is_fidelity(s) { s.source_color_hct().tone() }
                else if is_monochrome(s) { if s.is_dark { 85.0 } else { 25.0 } }
                else { if s.is_dark { 30.0 } else { 90.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(tdp), None,
        ))
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let pc = self.primary_container();
        let pc2 = pc.clone();
        Arc::new(DynamicColor::new(
            "on_primary_container".into(), Arc::new(|s| s.primary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(pc.clone()))),
            Some(Arc::new(move |s| {
                if is_fidelity(s) {
                    DynamicColor::foreground_tone((pc2.tone)(s), 4.5)
                } else if is_monochrome(s) { if s.is_dark { 0.0 } else { 100.0 } }
                else { if s.is_dark { 90.0 } else { 30.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None, None,
        ))
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        let inv = self.inverse_surface();
        Arc::new(DynamicColor::new(
            "inverse_primary".into(), Arc::new(|s| s.primary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(inv.clone()))),
            Some(Arc::new(|s| if s.is_dark { 40.0 } else { 80.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            None, None,
        ))
    }

    fn secondary(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let sc = self.secondary_container();
        let sc2 = sc.clone();
        let sec_stub = Arc::new(DynamicColor::new(
            "secondary".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(sc.clone(), sec_stub.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "secondary".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(tdp), None,
        ))
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let sec = self.secondary();
        Arc::new(DynamicColor::new(
            "on_secondary".into(), Arc::new(|s| s.secondary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(sec.clone()))),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 10.0 } else { 100.0 } }
                else { if s.is_dark { 20.0 } else { 100.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let sec_stub = Arc::new(DynamicColor::new(
            "secondary".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let sc_self = Arc::new(DynamicColor::new(
            "secondary_container".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| {
                let initial = if s.is_dark { 30.0 } else { 90.0 };
                if is_monochrome(s) { if s.is_dark { 30.0 } else { 85.0 } }
                else if !is_fidelity(s) { initial }
                else { find_desired_chroma_by_tone(s.secondary_palette.hue, s.secondary_palette.chroma, initial, !s.is_dark) }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None, None,
        ));
        let sc2 = sc_self.clone();
        let s2 = sec_stub.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(sc2.clone(), s2.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "secondary_container".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| {
                let initial = if s.is_dark { 30.0 } else { 90.0 };
                if is_monochrome(s) { if s.is_dark { 30.0 } else { 85.0 } }
                else if !is_fidelity(s) { initial }
                else { find_desired_chroma_by_tone(s.secondary_palette.hue, s.secondary_palette.chroma, initial, !s.is_dark) }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(tdp), None,
        ))
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let sc = self.secondary_container();
        let sc2 = sc.clone();
        Arc::new(DynamicColor::new(
            "on_secondary_container".into(), Arc::new(|s| s.secondary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(sc.clone()))),
            Some(Arc::new(move |s| {
                if is_monochrome(s) { if s.is_dark { 90.0 } else { 10.0 } }
                else if !is_fidelity(s) { if s.is_dark { 90.0 } else { 30.0 } }
                else { DynamicColor::foreground_tone((sc2.tone)(s), 4.5) }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None, None,
        ))
    }

    fn tertiary(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let tc = self.tertiary_container();
        let t_stub = Arc::new(DynamicColor::new(
            "tertiary".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, None,
            Some(Arc::new(|s| if is_monochrome(s) { if s.is_dark { 90.0 } else { 25.0 } } else { if s.is_dark { 80.0 } else { 40.0 } })),
            None, None, None, None,
        ));
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(tc.clone(), t_stub.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "tertiary".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { if s.is_dark { 90.0 } else { 25.0 } } else { if s.is_dark { 80.0 } else { 40.0 } })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(tdp), None,
        ))
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let t = self.tertiary();
        Arc::new(DynamicColor::new(
            "on_tertiary".into(), Arc::new(|s| s.tertiary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(t.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { if s.is_dark { 10.0 } else { 90.0 } } else { if s.is_dark { 20.0 } else { 100.0 } })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let t_stub = Arc::new(DynamicColor::new(
            "tertiary".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let tc_self = Arc::new(DynamicColor::new(
            "tertiary_container".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 60.0 } else { 49.0 } }
                else if !is_fidelity(s) { if s.is_dark { 30.0 } else { 90.0 } }
                else {
                    let proposed = s.tertiary_palette.get_hct(s.source_color_hct().tone());
                    DislikeAnalyzer::fix_if_disliked(proposed).tone()
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None, None,
        ));
        let tc2 = tc_self.clone();
        let ts2 = t_stub.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(tc2.clone(), ts2.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "tertiary_container".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 60.0 } else { 49.0 } }
                else if !is_fidelity(s) { if s.is_dark { 30.0 } else { 90.0 } }
                else {
                    let proposed = s.tertiary_palette.get_hct(s.source_color_hct().tone());
                    DislikeAnalyzer::fix_if_disliked(proposed).tone()
                }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(tdp), None,
        ))
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let tc = self.tertiary_container();
        let tc2 = tc.clone();
        Arc::new(DynamicColor::new(
            "on_tertiary_container".into(), Arc::new(|s| s.tertiary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(tc.clone()))),
            Some(Arc::new(move |s| {
                if is_monochrome(s) { if s.is_dark { 0.0 } else { 100.0 } }
                else if !is_fidelity(s) { if s.is_dark { 90.0 } else { 30.0 } }
                else { DynamicColor::foreground_tone((tc2.tone)(s), 4.5) }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None, None,
        ))
    }

    fn error(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let ec = self.error_container();
        let e_stub = Arc::new(DynamicColor::new(
            "error".into(), Arc::new(|s| s.error_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(ec.clone(), e_stub.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "error".into(), Arc::new(|s| s.error_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 7.0)))),
            Some(tdp), None,
        ))
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let e = self.error();
        Arc::new(DynamicColor::new(
            "on_error".into(), Arc::new(|s| s.error_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(e.clone()))),
            Some(Arc::new(|s| if s.is_dark { 20.0 } else { 100.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))),
            None, None,
        ))
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let e_stub = Arc::new(DynamicColor::new(
            "error".into(), Arc::new(|s| s.error_palette.clone()), true, None, None,
            Some(Arc::new(|s| if s.is_dark { 80.0 } else { 40.0 })),
            None, None, None, None,
        ));
        let ec_self = Arc::new(DynamicColor::new(
            "error_container".into(), Arc::new(|s| s.error_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            None, None,
        ));
        let ec2 = ec_self.clone();
        let es2 = e_stub.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(ec2.clone(), es2.clone(), 10.0, TonePolarity::RelativeLighter, false, DeltaConstraint::Nearer)));
        Arc::new(DynamicColor::new(
            "error_container".into(), Arc::new(|s| s.error_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if s.is_dark { 30.0 } else { 90.0 })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))),
            Some(tdp), None,
        ))
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let ec = self.error_container();
        Arc::new(DynamicColor::new(
            "on_error_container".into(), Arc::new(|s| s.error_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(ec.clone()))),
            Some(Arc::new(|s| {
                if is_monochrome(s) { if s.is_dark { 90.0 } else { 10.0 } }
                else { if s.is_dark { 90.0 } else { 30.0 } }
            })),
            None,
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))),
            None, None,
        ))
    }

    // ── Fixed colors ───────────────────────────────────────────────────

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let pfd = self.primary_fixed_dim();
        let pf_self = Arc::new(DynamicColor::new(
            "primary_fixed".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ));
        let pf2 = pf_self.clone(); let pfd2 = pfd.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(pf2.clone(), pfd2.clone(), 10.0, TonePolarity::Lighter, true, DeltaConstraint::Exact)));
        Arc::new(DynamicColor::new(
            "primary_fixed".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), Some(tdp), None,
        ))
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "primary_fixed_dim".into(), Arc::new(|s| s.primary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 30.0 } else { 80.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ))
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim(); let pf = self.primary_fixed();
        Arc::new(DynamicColor::new(
            "on_primary_fixed".into(), Arc::new(|s| s.primary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { 100.0 } else { 10.0 })),
            Some(Arc::new(move |_| Some(pf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))), None, None,
        ))
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let pfd = self.primary_fixed_dim(); let pf = self.primary_fixed();
        Arc::new(DynamicColor::new(
            "on_primary_fixed_variant".into(), Arc::new(|s| s.primary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(pfd.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { 90.0 } else { 30.0 })),
            Some(Arc::new(move |_| Some(pf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))), None, None,
        ))
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let sfd = self.secondary_fixed_dim();
        let sf_self = Arc::new(DynamicColor::new(
            "secondary_fixed".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| if is_monochrome(s) { 80.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ));
        let sf2 = sf_self.clone(); let sfd2 = sfd.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(sf2.clone(), sfd2.clone(), 10.0, TonePolarity::Lighter, true, DeltaConstraint::Exact)));
        Arc::new(DynamicColor::new(
            "secondary_fixed".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 80.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), Some(tdp), None,
        ))
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "secondary_fixed_dim".into(), Arc::new(|s| s.secondary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 70.0 } else { 80.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ))
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim(); let sf = self.secondary_fixed();
        Arc::new(DynamicColor::new(
            "on_secondary_fixed".into(), Arc::new(|s| s.secondary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            Some(Arc::new(|_| 10.0)),
            Some(Arc::new(move |_| Some(sf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))), None, None,
        ))
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let sfd = self.secondary_fixed_dim(); let sf = self.secondary_fixed();
        Arc::new(DynamicColor::new(
            "on_secondary_fixed_variant".into(), Arc::new(|s| s.secondary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(sfd.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { 25.0 } else { 30.0 })),
            Some(Arc::new(move |_| Some(sf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))), None, None,
        ))
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        let tfd = self.tertiary_fixed_dim();
        let tf_self = Arc::new(DynamicColor::new(
            "tertiary_fixed".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs.clone()),
            Some(Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ));
        let tf2 = tf_self.clone(); let tfd2 = tfd.clone();
        let tdp: crate::dynamiccolor::dynamic_color::DynamicColorFunction<Option<ToneDeltaPair>> =
            Arc::new(move |_| Some(ToneDeltaPair::new(tf2.clone(), tfd2.clone(), 10.0, TonePolarity::Lighter, true, DeltaConstraint::Exact)));
        Arc::new(DynamicColor::new(
            "tertiary_fixed".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 40.0 } else { 90.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), Some(tdp), None,
        ))
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let hs = Arc::new(|s: &DynamicScheme| Some(ColorSpec2021.highest_surface(s)));
        Arc::new(DynamicColor::new(
            "tertiary_fixed_dim".into(), Arc::new(|s| s.tertiary_palette.clone()), true, None, Some(hs),
            Some(Arc::new(|s| if is_monochrome(s) { 30.0 } else { 80.0 })),
            None, Some(Arc::new(|_| Some(ContrastCurve::new(1.0, 1.0, 3.0, 4.5)))), None, None,
        ))
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim(); let tf = self.tertiary_fixed();
        Arc::new(DynamicColor::new(
            "on_tertiary_fixed".into(), Arc::new(|s| s.tertiary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { 100.0 } else { 10.0 })),
            Some(Arc::new(move |_| Some(tf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(4.5, 7.0, 11.0, 21.0)))), None, None,
        ))
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let tfd = self.tertiary_fixed_dim(); let tf = self.tertiary_fixed();
        Arc::new(DynamicColor::new(
            "on_tertiary_fixed_variant".into(), Arc::new(|s| s.tertiary_palette.clone()), false, None,
            Some(Arc::new(move |_| Some(tfd.clone()))),
            Some(Arc::new(|s| if is_monochrome(s) { 90.0 } else { 30.0 })),
            Some(Arc::new(move |_| Some(tf.clone()))),
            Some(Arc::new(|_| Some(ContrastCurve::new(3.0, 4.5, 7.0, 11.0)))), None, None,
        ))
    }

    // ── Other ──────────────────────────────────────────────────────────

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        if scheme.is_dark { self.surface_bright() } else { self.surface_dim() }
    }

    // ── Color value calculations ───────────────────────────────────────

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        let tone = self.get_tone(scheme, color);
        (color.palette)(scheme).get_hct(tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        let decreasing_contrast = scheme.contrast_level < 0.0;
        let tone_delta_pair = color.tone_delta_pair.as_ref().and_then(|f| f(scheme));

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

            // 1st round: solve to min contrast for each
            if let (Some(bg_fn), Some(n_cc), Some(f_cc)) = (
                color.background.as_ref(),
                nearer.contrast_curve.as_ref().and_then(|f| f(scheme)),
                farther.contrast_curve.as_ref().and_then(|f| f(scheme)),
            ) {
                if let Some(bg_color) = bg_fn(scheme) {
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
            }

            if (f_tone - n_tone) * expansion_dir < delta {
                f_tone = (n_tone + delta * expansion_dir).clamp(0.0, 100.0);
                if (f_tone - n_tone) * expansion_dir < delta {
                    n_tone = (f_tone - delta * expansion_dir).clamp(0.0, 100.0);
                }
            }

            // Avoid the 50–59 awkward zone
            if (50.0..60.0).contains(&n_tone) {
                if expansion_dir > 0.0 {
                    n_tone = 60.0;
                    f_tone = f_tone.max(n_tone + delta * expansion_dir);
                } else {
                    n_tone = 49.0;
                    f_tone = f_tone.min(n_tone + delta * expansion_dir);
                }
            } else if (50.0..60.0).contains(&f_tone) {
                if stay_together {
                    if expansion_dir > 0.0 {
                        n_tone = 60.0;
                        f_tone = f_tone.max(n_tone + delta * expansion_dir);
                    } else {
                        n_tone = 49.0;
                        f_tone = f_tone.min(n_tone + delta * expansion_dir);
                    }
                } else {
                    f_tone = if expansion_dir > 0.0 { 60.0 } else { 49.0 };
                }
            }

            if am_nearer { n_tone } else { f_tone }
        } else {
            // Case 2: no tone delta pair; solve for self
            let mut answer = (color.tone)(scheme);
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
                answer = if Contrast::ratio_of_tones(49.0, bg_tone) >= desired_ratio { 49.0 } else { 60.0 };
            }

            let second_bg = color.second_background.as_ref().and_then(|f| f(scheme));
            let Some(bg2_color) = second_bg else { return answer; };

            // Case 3: dual backgrounds
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
                (Some(l), Some(d)) => d,
                (Some(l), None) => l,
                (None, Some(d)) => d,
                (None, None) => 0.0,
            }
        }
    }

    // ── Palette builders ───────────────────────────────────────────────

    fn get_primary_palette(&self, variant: Variant, src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => TonalPalette::from_hue_and_chroma(src.hue(), src.chroma()),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() - 50.0), 48.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 12.0),
            Variant::Rainbow => TonalPalette::from_hue_and_chroma(src.hue(), 48.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 36.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() + 240.0), 40.0),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(src.hue(), 200.0),
            v => panic!("{:?} variant not supported in Spec2021", v),
        }
    }

    fn get_secondary_palette(&self, variant: Variant, src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity =>
                TonalPalette::from_hue_and_chroma(src.hue(), (src.chroma() - 32.0).max(src.chroma() * 0.5)),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() - 50.0), 36.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 8.0),
            Variant::Rainbow => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(src,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0]), 24.0),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(src,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0]), 24.0),
            v => panic!("{:?} variant not supported in Spec2021", v),
        }
    }

    fn get_tertiary_palette(&self, variant: Variant, src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        match variant {
            Variant::Content => TonalPalette::from_hct(DislikeAnalyzer::fix_if_disliked(
                TemperatureCache::new(src.clone()).get_analogous_colors_with_options(3, 6)[2].clone())),
            Variant::Fidelity => TonalPalette::from_hct(DislikeAnalyzer::fix_if_disliked(
                TemperatureCache::new(src.clone()).complement())),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(src.hue(), 36.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::Rainbow | Variant::TonalSpot =>
                TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() + 60.0), 24.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(src,
                    &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                    &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0]), 32.0),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                DynamicScheme::get_rotated_hue(src,
                    &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                    &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0]), 32.0),
            v => panic!("{:?} variant not supported in Spec2021", v),
        }
    }

    fn get_neutral_palette(&self, variant: Variant, src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => TonalPalette::from_hue_and_chroma(src.hue(), src.chroma() / 8.0),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(src.hue(), 10.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 2.0),
            Variant::Rainbow => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 6.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() + 15.0), 8.0),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(src.hue(), 10.0),
            v => panic!("{:?} variant not supported in Spec2021", v),
        }
    }

    fn get_neutral_variant_palette(&self, variant: Variant, src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        match variant {
            Variant::Content | Variant::Fidelity => TonalPalette::from_hue_and_chroma(src.hue(), src.chroma() / 8.0 + 4.0),
            Variant::FruitSalad => TonalPalette::from_hue_and_chroma(src.hue(), 16.0),
            Variant::Monochrome => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::Neutral => TonalPalette::from_hue_and_chroma(src.hue(), 2.0),
            Variant::Rainbow => TonalPalette::from_hue_and_chroma(src.hue(), 0.0),
            Variant::TonalSpot => TonalPalette::from_hue_and_chroma(src.hue(), 8.0),
            Variant::Expressive => TonalPalette::from_hue_and_chroma(MathUtils::sanitize_degrees_double(src.hue() + 15.0), 12.0),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(src.hue(), 12.0),
            v => panic!("{:?} variant not supported in Spec2021", v),
        }
    }

    fn get_error_palette(&self, _variant: Variant, _src: &Hct, _dark: bool, _plat: Platform, _contrast: f64) -> TonalPalette {
        TonalPalette::from_hue_and_chroma(25.0, 84.0)
    }
}
