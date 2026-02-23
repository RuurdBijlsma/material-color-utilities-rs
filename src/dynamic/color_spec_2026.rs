use std::sync::Arc;

use crate::dynamic::color_spec::{ColorSpec, Platform, SpecVersion};
use crate::dynamic::color_spec_2025::ColorSpec2025;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, ToneDeltaPair, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

pub struct ColorSpec2026 {
    base: ColorSpec2025,
}

impl ColorSpec2026 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: ColorSpec2025::new(),
        }
    }

    fn get_contrast_curve(default_contrast: f64) -> ContrastCurve {
        match default_contrast {
            x if (x - 1.5).abs() < 1e-9 => ContrastCurve::new(1.5, 1.5, 3.0, 5.5),
            x if (x - 3.0).abs() < 1e-9 => ContrastCurve::new(3.0, 3.0, 4.5, 7.0),
            x if (x - 4.5).abs() < 1e-9 => ContrastCurve::new(4.5, 4.5, 7.0, 11.0),
            x if (x - 6.0).abs() < 1e-9 => ContrastCurve::new(6.0, 6.0, 7.0, 11.0),
            x if (x - 7.0).abs() < 1e-9 => ContrastCurve::new(7.0, 7.0, 11.0, 21.0),
            x if (x - 9.0).abs() < 1e-9 => ContrastCurve::new(9.0, 9.0, 11.0, 21.0),
            x if (x - 11.0).abs() < 1e-9 => ContrastCurve::new(11.0, 11.0, 21.0, 21.0),
            x if (x - 21.0).abs() < 1e-9 => ContrastCurve::new(21.0, 21.0, 21.0, 21.0),
            _ => ContrastCurve::new(default_contrast, default_contrast, 7.0, 21.0),
        }
    }

    fn find_best_tone_for_chroma(
        hue: f64,
        chroma: f64,
        start_tone: f64,
        by_decreasing_tone: bool,
    ) -> f64 {
        let mut tone = start_tone;
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

    fn highest_surface_background(
        &self,
    ) -> Arc<dyn Fn(&DynamicScheme) -> Option<Arc<DynamicColor>> + Send + Sync> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(move |scheme| {
            if scheme.platform == Platform::Phone {
                if scheme.is_dark {
                    Some(surface_bright.clone())
                } else {
                    Some(surface_dim.clone())
                }
            } else {
                Some(surface_container_high.clone())
            }
        })
    }
}

impl Default for ColorSpec2026 {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSpec for ColorSpec2026 {
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

    fn background(&self) -> Arc<DynamicColor> {
        self.base.background()
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        self.base.on_background()
    }

    fn surface(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 4.0 } else { 98.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_dim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 1.0 } else { 1.7 }
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 4.0 } else { 87.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_bright(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_bright".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 1.7 } else { 1.0 }
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 18.0 } else { 98.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_container_lowest(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_container_lowest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 0.0 } else { 100.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_container_low(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_container_low".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    1.25
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 6.0 } else { 96.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_container".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    1.4
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 9.0 } else { 94.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_container_high".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    1.5
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 12.0 } else { 92.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "surface_container_highest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    1.7
                } else {
                    0.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf {
                    if scheme.is_dark { 15.0 } else { 90.0 }
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf { 1.7 } else { 0.0 }
            })),
            Some(self.highest_surface_background()),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(Self::get_contrast_curve(if scheme.is_dark { 11.0 } else { 9.0 }))
            })),
            None,
            None,
        );
        self.base
            .on_surface()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        self.base.surface_variant()
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "on_surface_variant".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf { 1.7 } else { 0.0 }
            })),
            Some(self.highest_surface_background()),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(Self::get_contrast_curve(if scheme.is_dark { 6.0 } else { 4.5 }))
            })),
            None,
            None,
        );
        self.base
            .on_surface_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn inverse_surface(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "inverse_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf { 1.7 } else { 0.0 }
            })),
            None,
            Some(Arc::new(|scheme| if scheme.is_dark { 98.0 } else { 4.0 })),
            None,
            None,
            None,
            None,
        );
        self.base
            .inverse_surface()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let inverse_surface = self.inverse_surface();
        let color_2026 = DynamicColor::new(
            "inverse_on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(inverse_surface.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .inverse_on_surface()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "outline".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf { 1.7 } else { 0.0 }
            })),
            Some(self.highest_surface_background()),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(3.0)))),
            None,
            None,
        );
        self.base
            .outline()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "outline_variant".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Cmf { 1.7 } else { 0.0 }
            })),
            Some(self.highest_surface_background()),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(1.5)))),
            None,
            None,
        );
        self.base
            .outline_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
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

    fn primary(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.source_color_hct().chroma() <= 12.0 {
                    if scheme.is_dark { 80.0 } else { 40.0 }
                } else {
                    scheme.source_color_hct().tone()
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .primary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.primary_dim()
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let color_2026 = DynamicColor::new(
            "on_primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(primary.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_primary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let primary_container_stub = Arc::new(DynamicColor::new(
            "primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if !scheme.is_dark && scheme.source_color_hct().chroma() <= 12.0 {
                    90.0
                } else if scheme.source_color_hct().tone() > 55.0 {
                    scheme.source_color_hct().tone().clamp(61.0, 90.0)
                } else {
                    scheme.source_color_hct().tone().clamp(30.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let primary_container_for_delta = primary_container_stub.clone();
        let color_2026 = DynamicColor::new(
            "primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if !scheme.is_dark && scheme.source_color_hct().chroma() <= 12.0 {
                    90.0
                } else if scheme.source_color_hct().tone() > 55.0 {
                    scheme.source_color_hct().tone().clamp(61.0, 90.0)
                } else {
                    scheme.source_color_hct().tone().clamp(30.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    primary_container_for_delta.clone(),
                    primary.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .primary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let primary_container = self.primary_container();
        let color_2026 = DynamicColor::new(
            "on_primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(primary_container.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_primary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        self.base.inverse_primary()
    }

    fn secondary(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.secondary_palette, 0.0, 100.0)
                } else {
                    Self::t_max_c(&scheme.secondary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .secondary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.secondary_dim()
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let secondary = self.secondary();
        let color_2026 = DynamicColor::new(
            "on_secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(secondary.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_secondary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let secondary = self.secondary();
        let secondary_container_stub = Arc::new(DynamicColor::new(
            "secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.secondary_palette, 20.0, 49.0)
                } else {
                    Self::t_max_c(&scheme.secondary_palette, 61.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let secondary_container_for_delta = secondary_container_stub.clone();
        let color_2026 = DynamicColor::new(
            "secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.secondary_palette, 20.0, 49.0)
                } else {
                    Self::t_max_c(&scheme.secondary_palette, 61.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    secondary_container_for_delta.clone(),
                    secondary.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .secondary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let secondary_container = self.secondary_container();
        let color_2026 = DynamicColor::new(
            "on_secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(secondary_container.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_secondary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn tertiary(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                scheme
                    .source_color_hct_list
                    .get(1)
                    .map_or(scheme.source_color_hct().tone(), Hct::tone)
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .tertiary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.tertiary_dim()
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let tertiary = self.tertiary();
        let color_2026 = DynamicColor::new(
            "on_tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(tertiary.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let tertiary = self.tertiary();
        let tertiary_container_stub = Arc::new(DynamicColor::new(
            "tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                let secondary_source = scheme
                    .source_color_hct_list
                    .get(1)
                    .unwrap_or(scheme.source_color_hct());
                if secondary_source.tone() > 55.0 {
                    secondary_source.tone().clamp(61.0, 90.0)
                } else {
                    secondary_source.tone().clamp(20.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let tertiary_container_for_delta = tertiary_container_stub.clone();
        let color_2026 = DynamicColor::new(
            "tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                let secondary_source = scheme
                    .source_color_hct_list
                    .get(1)
                    .unwrap_or(scheme.source_color_hct());
                if secondary_source.tone() > 55.0 {
                    secondary_source.tone().clamp(61.0, 90.0)
                } else {
                    secondary_source.tone().clamp(20.0, 49.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    tertiary_container_for_delta.clone(),
                    tertiary.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .tertiary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let tertiary_container = self.tertiary_container();
        let color_2026 = DynamicColor::new(
            "on_tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(tertiary_container.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn error(&self) -> Arc<DynamicColor> {
        let color_2026 = DynamicColor::new(
            "error".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                Self::t_max_c(&scheme.error_palette, 0.0, 100.0, 1.0)
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .error()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        self.base.error_dim()
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let error = self.error();
        let color_2026 = DynamicColor::new(
            "on_error".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(error.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_error()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let error = self.error();
        let error_container_stub = Arc::new(DynamicColor::new(
            "error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.error_palette, 0.0, 100.0)
                } else {
                    Self::t_max_c(&scheme.error_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let error_container_for_delta = error_container_stub.clone();
        let color_2026 = DynamicColor::new(
            "error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    Self::t_min_c(&scheme.error_palette, 0.0, 100.0)
                } else {
                    Self::t_max_c(&scheme.error_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    error_container_for_delta.clone(),
                    error.clone(),
                    5.0,
                    TonePolarity::RelativeLighter,
                    false,
                    DeltaConstraint::Farther,
                ))
            })),
            None,
        );
        self.base
            .error_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let error_container = self.error_container();
        let color_2026 = DynamicColor::new(
            "on_error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(error_container.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(6.0)))),
            None,
            None,
        );
        self.base
            .on_error_container()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let primary_container = self.primary_container();
        let color_2026 = DynamicColor::new(
            "primary_fixed".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                primary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let primary_fixed = self.primary_fixed();
        let primary_fixed_dim_stub = Arc::new(DynamicColor::new(
            "primary_fixed_dim".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| primary_fixed.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let primary_fixed_for_delta = self.primary_fixed();
        let primary_fixed_dim_for_delta = primary_fixed_dim_stub.clone();
        let primary_fixed_for_tone = self.primary_fixed();
        let color_2026 = DynamicColor::new(
            "primary_fixed_dim".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| primary_fixed_for_tone.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    primary_fixed_dim_for_delta.clone(),
                    primary_fixed_for_delta.clone(),
                    5.0,
                    TonePolarity::Darker,
                    false,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .primary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let primary_fixed_dim = self.primary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_primary_fixed".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(primary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_primary_fixed()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let primary_fixed_dim = self.primary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_primary_fixed_variant".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(primary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_primary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let secondary_container = self.secondary_container();
        let color_2026 = DynamicColor::new(
            "secondary_fixed".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                secondary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let secondary_fixed = self.secondary_fixed();
        let secondary_fixed_dim_stub = Arc::new(DynamicColor::new(
            "secondary_fixed_dim".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| secondary_fixed.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let secondary_fixed_for_delta = self.secondary_fixed();
        let secondary_fixed_dim_for_delta = secondary_fixed_dim_stub.clone();
        let secondary_fixed_for_tone = self.secondary_fixed();
        let color_2026 = DynamicColor::new(
            "secondary_fixed_dim".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| secondary_fixed_for_tone.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    secondary_fixed_dim_for_delta.clone(),
                    secondary_fixed_for_delta.clone(),
                    5.0,
                    TonePolarity::Darker,
                    false,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .secondary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let secondary_fixed_dim = self.secondary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_secondary_fixed".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(secondary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_secondary_fixed()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let secondary_fixed_dim = self.secondary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_secondary_fixed_variant".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(secondary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_secondary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tertiary_container = self.tertiary_container();
        let color_2026 = DynamicColor::new(
            "tertiary_fixed".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                tertiary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
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
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let tertiary_fixed = self.tertiary_fixed();
        let tertiary_fixed_dim_stub = Arc::new(DynamicColor::new(
            "tertiary_fixed_dim".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| tertiary_fixed.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ));
        let tertiary_fixed_for_delta = self.tertiary_fixed();
        let tertiary_fixed_dim_for_delta = tertiary_fixed_dim_stub.clone();
        let tertiary_fixed_for_tone = self.tertiary_fixed();
        let color_2026 = DynamicColor::new(
            "tertiary_fixed_dim".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(self.highest_surface_background()),
            Some(Arc::new(move |scheme| tertiary_fixed_for_tone.get_tone(scheme))),
            None,
            Some(Arc::new(|scheme| {
                if scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            Some(Arc::new(move |_| {
                Some(ToneDeltaPair::new(
                    tertiary_fixed_dim_for_delta.clone(),
                    tertiary_fixed_for_delta.clone(),
                    5.0,
                    TonePolarity::Darker,
                    false,
                    DeltaConstraint::Exact,
                ))
            })),
            None,
        );
        self.base
            .tertiary_fixed_dim()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tertiary_fixed_dim = self.tertiary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_tertiary_fixed".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(tertiary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(7.0)))),
            None,
            None,
        );
        self.base
            .on_tertiary_fixed()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let tertiary_fixed_dim = self.tertiary_fixed_dim();
        let color_2026 = DynamicColor::new(
            "on_tertiary_fixed_variant".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(tertiary_fixed_dim.clone()))),
            None,
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        );
        self.base
            .on_tertiary_fixed_variant()
            .extend_spec_version(SpecVersion::Spec2026, &color_2026)
    }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        self.base.highest_surface(scheme)
    }

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        self.base.get_hct(scheme, color)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        self.base.get_tone(scheme, color)
    }

    fn get_primary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base.get_primary_palette(
            variant,
            source_color_hct,
            is_dark,
            platform,
            contrast_level,
        )
    }

    fn get_secondary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base.get_secondary_palette(
            variant,
            source_color_hct,
            is_dark,
            platform,
            contrast_level,
        )
    }

    fn get_tertiary_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base.get_tertiary_palette(
            variant,
            source_color_hct,
            is_dark,
            platform,
            contrast_level,
        )
    }

    fn get_neutral_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base
            .get_neutral_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }

    fn get_neutral_variant_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base.get_neutral_variant_palette(
            variant,
            source_color_hct,
            is_dark,
            platform,
            contrast_level,
        )
    }

    fn get_error_palette(
        &self,
        variant: Variant,
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
        contrast_level: f64,
    ) -> TonalPalette {
        self.base
            .get_error_palette(variant, source_color_hct, is_dark, platform, contrast_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheme::scheme_cmf::SchemeCmf;
    use crate::utils::color_utils::Argb;

    #[test]
    fn cmf_surface_tones_match_2026_rules() {
        let source = Hct::from_int(Argb(0xFF4285F4));
        let spec = ColorSpec2026::new();

        let light = SchemeCmf::new_with_platform_and_spec(
            source,
            false,
            0.0,
            SpecVersion::Spec2026,
            Platform::Phone,
        );
        let dark = SchemeCmf::new_with_platform_and_spec(
            source,
            true,
            0.0,
            SpecVersion::Spec2026,
            Platform::Phone,
        );

        assert_eq!(spec.surface().get_tone(&light), 98.0);
        assert_eq!(spec.surface().get_tone(&dark), 4.0);
        assert_eq!(spec.surface_dim().get_tone(&light), 87.0);
        assert_eq!(spec.surface_bright().get_tone(&dark), 18.0);
    }

    #[test]
    fn tertiary_uses_second_source_tone_when_present() {
        let primary = Hct::from_int(Argb(0xFF4285F4));
        let secondary = Hct::from_int(Argb(0xFFB00020));
        let spec = ColorSpec2026::new();

        let scheme = SchemeCmf::new_with_list_and_platform_and_spec(
            vec![primary, secondary],
            false,
            0.0,
            SpecVersion::Spec2026,
            Platform::Phone,
        );
        let tertiary_tone = spec.tertiary().get_tone(&scheme);
        assert!((tertiary_tone - secondary.tone()).abs() < 1e-6);
    }
}
