use std::sync::Arc;

use crate::contrast::contrast_utils::Contrast;
use crate::dynamic::color_spec::{ColorSpec, Platform, SpecVersion};
use crate::dynamic::color_spec_2021::ColorSpec2021;
use crate::dynamic::contrast_curve::ContrastCurve;
use crate::dynamic::dynamic_color::DynamicColor;
use crate::dynamic::dynamic_scheme::DynamicScheme;
use crate::dynamic::tone_delta_pair::{DeltaConstraint, TonePolarity};
use crate::dynamic::variant::Variant;
use crate::hct::hct_color::Hct;
use crate::palettes::tonal_palette::TonalPalette;

pub struct ColorSpec2025 {
    base: ColorSpec2021,
}

impl ColorSpec2025 {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            base: ColorSpec2021::new(),
        }
    }

    fn get_expressive_neutral_hue(source_color_hct: &Hct) -> f64 {
        DynamicScheme::get_rotated_hue(
            source_color_hct,
            &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
            &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
        )
    }

    fn get_expressive_neutral_chroma(
        source_color_hct: &Hct,
        is_dark: bool,
        platform: Platform,
    ) -> f64 {
        let neutral_hue = Self::get_expressive_neutral_hue(source_color_hct);
        if platform == Platform::Phone {
            if is_dark {
                if Hct::is_yellow(neutral_hue) {
                    6.0
                } else {
                    14.0
                }
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
        if platform == Platform::Phone || Hct::is_blue(neutral_hue) {
            28.0
        } else {
            20.0
        }
    }

    fn copy_with_name(color: &Arc<DynamicColor>, name: &str) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            name.to_string(),
            color.palette.clone(),
            color.is_background,
            color.chroma_multiplier.clone(),
            color.background.clone(),
            Some(color.tone.clone()),
            color.second_background.clone(),
            color.contrast_curve.clone(),
            color.tone_delta_pair.clone(),
            color.opacity.clone(),
        ))
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

    fn neutral_content_chroma_multiplier(scheme: &DynamicScheme) -> f64 {
        if scheme.platform == Platform::Phone {
            match scheme.variant {
                Variant::Neutral => 2.2,
                Variant::TonalSpot => 1.7,
                Variant::Expressive => {
                    if Hct::is_yellow(scheme.neutral_palette.hue) {
                        if scheme.is_dark { 3.0 } else { 2.3 }
                    } else {
                        1.6
                    }
                }
                _ => 1.0,
            }
        } else {
            1.0
        }
    }
}

impl Default for ColorSpec2025 {
    fn default() -> Self {
        Self::new()
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

    fn background(&self) -> Arc<DynamicColor> {
        Self::copy_with_name(&self.surface(), "background")
    }

    fn on_background(&self) -> Arc<DynamicColor> {
        let on_surface = self.on_surface();
        let on_surface_for_tone = on_surface.clone();
        Arc::new(DynamicColor::new(
            "on_background".into(),
            on_surface.palette.clone(),
            on_surface.is_background,
            on_surface.chroma_multiplier.clone(),
            on_surface.background.clone(),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Watch {
                    100.0
                } else {
                    on_surface_for_tone.get_tone(scheme)
                }
            })),
            on_surface.second_background.clone(),
            on_surface.contrast_curve.clone(),
            on_surface.tone_delta_pair.clone(),
            on_surface.opacity.clone(),
        ))
    }

    fn surface(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        4.0
                    } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                        99.0
                    } else if scheme.variant == Variant::Vibrant {
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
        ))
    }

    fn surface_dim(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_dim".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if !scheme.is_dark {
                    match scheme.variant {
                        Variant::Neutral => 2.5,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(scheme.neutral_palette.hue) {
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
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    4.0
                } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                    90.0
                } else if scheme.variant == Variant::Vibrant {
                    85.0
                } else {
                    87.0
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
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    match scheme.variant {
                        Variant::Neutral => 2.5,
                        Variant::TonalSpot => 1.7,
                        Variant::Expressive => {
                            if Hct::is_yellow(scheme.neutral_palette.hue) {
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
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    18.0
                } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                    99.0
                } else if scheme.variant == Variant::Vibrant {
                    97.0
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
            Some(Arc::new(|s| if s.is_dark { 0.0 } else { 100.0 })),
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
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    match scheme.variant {
                        Variant::Neutral => 1.3,
                        Variant::TonalSpot => 1.25,
                        Variant::Expressive => {
                            if Hct::is_yellow(scheme.neutral_palette.hue) {
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
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        6.0
                    } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                        98.0
                    } else if scheme.variant == Variant::Vibrant {
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
        ))
    }

    fn surface_container(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    match scheme.variant {
                        Variant::Neutral => 1.6,
                        Variant::TonalSpot => 1.4,
                        Variant::Expressive => {
                            if Hct::is_yellow(scheme.neutral_palette.hue) {
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
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        9.0
                    } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                        96.0
                    } else if scheme.variant == Variant::Vibrant {
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
        ))
    }

    fn surface_container_high(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_high".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    match scheme.variant {
                        Variant::Neutral => 1.9,
                        Variant::TonalSpot => 1.5,
                        Variant::Expressive => {
                            if Hct::is_yellow(scheme.neutral_palette.hue) {
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
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        12.0
                    } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                        94.0
                    } else if scheme.variant == Variant::Vibrant {
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
        ))
    }

    fn surface_container_highest(&self) -> Arc<DynamicColor> {
        Arc::new(DynamicColor::new(
            "surface_container_highest".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            true,
            Some(Arc::new(|scheme| match scheme.variant {
                Variant::Neutral => 2.2,
                Variant::TonalSpot => 1.7,
                Variant::Expressive => {
                    if Hct::is_yellow(scheme.neutral_palette.hue) {
                        2.3
                    } else {
                        1.6
                    }
                }
                Variant::Vibrant => 1.29,
                _ => 1.0,
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.is_dark {
                    15.0
                } else if Hct::is_yellow(scheme.neutral_palette.hue) {
                    92.0
                } else if scheme.variant == Variant::Vibrant {
                    88.0
                } else {
                    90.0
                }
            })),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_surface(&self) -> Arc<DynamicColor> {
        let surface_bright_tone = self.surface_bright();
        let surface_dim_tone = self.surface_dim();
        let surface_container_high_tone = self.surface_container_high();
        let surface_bright_bg = surface_bright_tone.clone();
        let surface_dim_bg = surface_dim_tone.clone();
        let surface_container_high_bg = surface_container_high_tone.clone();
        Arc::new(DynamicColor::new(
            "on_surface".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(Self::neutral_content_chroma_multiplier)),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright_bg.clone())
                    } else {
                        Some(surface_dim_bg.clone())
                    }
                } else {
                    Some(surface_container_high_bg.clone())
                }
            })),
            Some(Arc::new(move |scheme| {
                if scheme.variant == Variant::Vibrant {
                    Self::t_max_c(&scheme.neutral_palette, 0.0, 100.0, 1.1)
                } else if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        surface_bright_tone.get_tone(scheme)
                    } else {
                        surface_dim_tone.get_tone(scheme)
                    }
                } else {
                    surface_container_high_tone.get_tone(scheme)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.is_dark && scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(11.0)
                } else {
                    Self::get_contrast_curve(9.0)
                })
            })),
            None,
            None,
        ))
    }

    fn surface_variant(&self) -> Arc<DynamicColor> {
        Self::copy_with_name(&self.surface_container_highest(), "surface_variant")
    }

    fn on_surface_variant(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "on_surface_variant".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(Self::neutral_content_chroma_multiplier)),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Self::get_contrast_curve(6.0)
                    } else {
                        Self::get_contrast_curve(4.5)
                    }
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
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
            Some(Arc::new(|s| if s.is_dark { 98.0 } else { 4.0 })),
            None,
            None,
            None,
            None,
        ))
    }

    fn inverse_on_surface(&self) -> Arc<DynamicColor> {
        let inverse_surface = self.inverse_surface();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn outline(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "outline".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(Self::neutral_content_chroma_multiplier)),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(3.0)
                } else {
                    Self::get_contrast_curve(4.5)
                })
            })),
            None,
            None,
        ))
    }

    fn outline_variant(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "outline_variant".into(),
            Arc::new(|s| s.neutral_palette.clone()),
            false,
            Some(Arc::new(Self::neutral_content_chroma_multiplier)),
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(1.5)
                } else {
                    Self::get_contrast_curve(3.0)
                })
            })),
            None,
            None,
        ))
    }
    fn shadow(&self) -> Arc<DynamicColor> {
        self.base.shadow()
    }
    fn scrim(&self) -> Arc<DynamicColor> {
        self.base.scrim()
    }
    fn surface_tint(&self) -> Arc<DynamicColor> {
        Self::copy_with_name(&self.primary(), "surface_tint")
    }

    fn primary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Neutral {
                    if scheme.platform == Platform::Phone {
                        if scheme.is_dark { 80.0 } else { 40.0 }
                    } else {
                        90.0
                    }
                } else if scheme.variant == Variant::TonalSpot {
                    if scheme.platform == Platform::Phone {
                        if scheme.is_dark {
                            80.0
                        } else {
                            Self::t_max_c(&scheme.primary_palette, 0.0, 100.0, 1.0)
                        }
                    } else {
                        Self::t_max_c(&scheme.primary_palette, 0.0, 90.0, 1.0)
                    }
                } else if scheme.variant == Variant::Expressive {
                    if scheme.platform == Platform::Phone {
                        Self::t_max_c(
                            &scheme.primary_palette,
                            0.0,
                            if Hct::is_yellow(scheme.primary_palette.hue) {
                                25.0
                            } else if Hct::is_cyan(scheme.primary_palette.hue) {
                                88.0
                            } else {
                                98.0
                            },
                            1.0,
                        )
                    } else {
                        Self::t_max_c(&scheme.primary_palette, 0.0, 100.0, 1.0)
                    }
                } else if scheme.platform == Platform::Phone {
                    Self::t_max_c(
                        &scheme.primary_palette,
                        0.0,
                        if Hct::is_cyan(scheme.primary_palette.hue) {
                            88.0
                        } else {
                            98.0
                        },
                        1.0,
                    )
                } else {
                    Self::t_max_c(&scheme.primary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(4.5)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn primary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();
        Some(Arc::new(DynamicColor::new(
            "primary_dim".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(|scheme| match scheme.variant {
                Variant::Neutral => 85.0,
                Variant::TonalSpot => Self::t_max_c(&scheme.primary_palette, 0.0, 90.0, 1.0),
                _ => Self::t_max_c(&scheme.primary_palette, 0.0, 100.0, 1.0),
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        )))
    }

    fn on_primary(&self) -> Arc<DynamicColor> {
        let primary = self.primary();
        let primary_dim = self.primary_dim().expect("primary_dim exists in spec 2025");
        Arc::new(DynamicColor::new(
            "on_primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    Some(primary.clone())
                } else {
                    Some(primary_dim.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn primary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    30.0
                } else if scheme.variant == Variant::Neutral {
                    if scheme.is_dark { 30.0 } else { 90.0 }
                } else if scheme.variant == Variant::TonalSpot {
                    if scheme.is_dark {
                        Self::t_max_c(&scheme.primary_palette, 35.0, 93.0, 1.0)
                    } else {
                        Self::t_max_c(&scheme.primary_palette, 0.0, 90.0, 1.0)
                    }
                } else if scheme.variant == Variant::Expressive {
                    if scheme.is_dark {
                        Self::t_max_c(&scheme.primary_palette, 30.0, 93.0, 1.0)
                    } else {
                        Self::t_max_c(
                            &scheme.primary_palette,
                            78.0,
                            if Hct::is_cyan(scheme.primary_palette.hue) {
                                88.0
                            } else {
                                90.0
                            },
                            1.0,
                        )
                    }
                } else if scheme.is_dark {
                    Self::t_max_c(&scheme.primary_palette, 66.0, 93.0, 1.0)
                } else {
                    Self::t_max_c(
                        &scheme.primary_palette,
                        66.0,
                        if Hct::is_cyan(scheme.primary_palette.hue) {
                            88.0
                        } else {
                            93.0
                        },
                        1.0,
                    )
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn on_primary_container(&self) -> Arc<DynamicColor> {
        let primary_container = self.primary_container();
        Arc::new(DynamicColor::new(
            "on_primary_container".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(primary_container.clone()))),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn inverse_primary(&self) -> Arc<DynamicColor> {
        let inverse_surface = self.inverse_surface();
        Arc::new(DynamicColor::new(
            "inverse_primary".into(),
            Arc::new(|s| s.primary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(inverse_surface.clone()))),
            Some(Arc::new(|scheme| {
                Self::t_max_c(&scheme.primary_palette, 0.0, 100.0, 1.0)
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn secondary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    if scheme.variant == Variant::Neutral {
                        90.0
                    } else {
                        Self::t_max_c(&scheme.secondary_palette, 0.0, 90.0, 1.0)
                    }
                } else if scheme.variant == Variant::Neutral {
                    if scheme.is_dark {
                        Self::t_min_c(&scheme.secondary_palette, 0.0, 98.0)
                    } else {
                        Self::t_max_c(&scheme.secondary_palette, 0.0, 100.0, 1.0)
                    }
                } else if scheme.variant == Variant::Vibrant {
                    Self::t_max_c(
                        &scheme.secondary_palette,
                        0.0,
                        if scheme.is_dark { 90.0 } else { 98.0 },
                        1.0,
                    )
                } else if scheme.is_dark {
                    80.0
                } else {
                    Self::t_max_c(&scheme.secondary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(4.5)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn secondary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();
        Some(Arc::new(DynamicColor::new(
            "secondary_dim".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::Neutral {
                    85.0
                } else {
                    Self::t_max_c(&scheme.secondary_palette, 0.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        )))
    }

    fn on_secondary(&self) -> Arc<DynamicColor> {
        let secondary = self.secondary();
        let secondary_dim = self
            .secondary_dim()
            .expect("secondary_dim exists in spec 2025");
        Arc::new(DynamicColor::new(
            "on_secondary".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    Some(secondary.clone())
                } else {
                    Some(secondary_dim.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn secondary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    30.0
                } else if scheme.variant == Variant::Vibrant {
                    if scheme.is_dark {
                        Self::t_min_c(&scheme.secondary_palette, 30.0, 40.0)
                    } else {
                        Self::t_max_c(&scheme.secondary_palette, 84.0, 90.0, 1.0)
                    }
                } else if scheme.variant == Variant::Expressive {
                    if scheme.is_dark {
                        15.0
                    } else {
                        Self::t_max_c(&scheme.secondary_palette, 90.0, 95.0, 1.0)
                    }
                } else if scheme.is_dark {
                    25.0
                } else {
                    90.0
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn on_secondary_container(&self) -> Arc<DynamicColor> {
        let secondary_container = self.secondary_container();
        Arc::new(DynamicColor::new(
            "on_secondary_container".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(secondary_container.clone()))),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn tertiary(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    if scheme.variant == Variant::TonalSpot {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 90.0, 1.0)
                    } else {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 100.0, 1.0)
                    }
                } else if scheme.variant == Variant::Expressive
                    || scheme.variant == Variant::Vibrant
                {
                    Self::t_max_c(
                        &scheme.tertiary_palette,
                        0.0,
                        if Hct::is_cyan(scheme.tertiary_palette.hue) {
                            88.0
                        } else if scheme.is_dark {
                            98.0
                        } else {
                            100.0
                        },
                        1.0,
                    )
                } else if scheme.is_dark {
                    Self::t_max_c(&scheme.tertiary_palette, 0.0, 98.0, 1.0)
                } else {
                    Self::t_max_c(&scheme.tertiary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(4.5)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn tertiary_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();
        Some(Arc::new(DynamicColor::new(
            "tertiary_dim".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(|scheme| {
                if scheme.variant == Variant::TonalSpot {
                    Self::t_max_c(&scheme.tertiary_palette, 0.0, 90.0, 1.0)
                } else {
                    Self::t_max_c(&scheme.tertiary_palette, 0.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        )))
    }

    fn on_tertiary(&self) -> Arc<DynamicColor> {
        let tertiary = self.tertiary();
        let tertiary_dim = self
            .tertiary_dim()
            .expect("tertiary_dim exists in spec 2025");
        Arc::new(DynamicColor::new(
            "on_tertiary".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    Some(tertiary.clone())
                } else {
                    Some(tertiary_dim.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn tertiary_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    if scheme.variant == Variant::TonalSpot {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 90.0, 1.0)
                    } else {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 100.0, 1.0)
                    }
                } else if scheme.variant == Variant::Neutral {
                    if scheme.is_dark {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 93.0, 1.0)
                    } else {
                        Self::t_max_c(&scheme.tertiary_palette, 0.0, 96.0, 1.0)
                    }
                } else if scheme.variant == Variant::TonalSpot {
                    Self::t_max_c(
                        &scheme.tertiary_palette,
                        0.0,
                        if scheme.is_dark { 93.0 } else { 100.0 },
                        1.0,
                    )
                } else if scheme.variant == Variant::Expressive {
                    Self::t_max_c(
                        &scheme.tertiary_palette,
                        75.0,
                        if Hct::is_cyan(scheme.tertiary_palette.hue) {
                            88.0
                        } else if scheme.is_dark {
                            93.0
                        } else {
                            100.0
                        },
                        1.0,
                    )
                } else if scheme.is_dark {
                    Self::t_max_c(&scheme.tertiary_palette, 0.0, 93.0, 1.0)
                } else {
                    Self::t_max_c(&scheme.tertiary_palette, 72.0, 100.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn on_tertiary_container(&self) -> Arc<DynamicColor> {
        let tertiary_container = self.tertiary_container();
        Arc::new(DynamicColor::new(
            "on_tertiary_container".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(tertiary_container.clone()))),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn error(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        let surface_container_high = self.surface_container_high();
        Arc::new(DynamicColor::new(
            "error".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    Some(surface_container_high.clone())
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Self::t_min_c(&scheme.error_palette, 0.0, 98.0)
                    } else {
                        Self::t_max_c(&scheme.error_palette, 0.0, 100.0, 1.0)
                    }
                } else {
                    Self::t_min_c(&scheme.error_palette, 0.0, 100.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(4.5)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn error_dim(&self) -> Option<Arc<DynamicColor>> {
        let surface_container_high = self.surface_container_high();
        Some(Arc::new(DynamicColor::new(
            "error_dim".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(move |_| Some(surface_container_high.clone()))),
            Some(Arc::new(|scheme| {
                Self::t_min_c(&scheme.error_palette, 0.0, 100.0)
            })),
            None,
            Some(Arc::new(|_| Some(Self::get_contrast_curve(4.5)))),
            None,
            None,
        )))
    }

    fn on_error(&self) -> Arc<DynamicColor> {
        let error = self.error();
        let error_dim = self.error_dim().expect("error_dim exists in spec 2025");
        Arc::new(DynamicColor::new(
            "on_error".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    Some(error.clone())
                } else {
                    Some(error_dim.clone())
                }
            })),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(6.0)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn error_container(&self) -> Arc<DynamicColor> {
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Watch {
                    30.0
                } else if scheme.is_dark {
                    Self::t_min_c(&scheme.error_palette, 30.0, 93.0)
                } else {
                    Self::t_max_c(&scheme.error_palette, 0.0, 90.0, 1.0)
                }
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn on_error_container(&self) -> Arc<DynamicColor> {
        let error_container = self.error_container();
        Arc::new(DynamicColor::new(
            "on_error_container".into(),
            Arc::new(|s| s.error_palette.clone()),
            false,
            None,
            Some(Arc::new(move |_| Some(error_container.clone()))),
            None,
            None,
            Some(Arc::new(|scheme| {
                Some(if scheme.platform == Platform::Phone {
                    Self::get_contrast_curve(4.5)
                } else {
                    Self::get_contrast_curve(7.0)
                })
            })),
            None,
            None,
        ))
    }

    fn primary_fixed(&self) -> Arc<DynamicColor> {
        let primary_container = self.primary_container();
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "primary_fixed".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                primary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn primary_fixed_dim(&self) -> Arc<DynamicColor> {
        let primary_fixed = self.primary_fixed();
        Arc::new(DynamicColor::new(
            "primary_fixed_dim".into(),
            Arc::new(|s| s.primary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(move |scheme| primary_fixed.get_tone(scheme))),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_primary_fixed(&self) -> Arc<DynamicColor> {
        let primary_fixed_dim = self.primary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn on_primary_fixed_variant(&self) -> Arc<DynamicColor> {
        let primary_fixed_dim = self.primary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn secondary_fixed(&self) -> Arc<DynamicColor> {
        let secondary_container = self.secondary_container();
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "secondary_fixed".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                secondary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn secondary_fixed_dim(&self) -> Arc<DynamicColor> {
        let secondary_fixed = self.secondary_fixed();
        Arc::new(DynamicColor::new(
            "secondary_fixed_dim".into(),
            Arc::new(|s| s.secondary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(move |scheme| secondary_fixed.get_tone(scheme))),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_secondary_fixed(&self) -> Arc<DynamicColor> {
        let secondary_fixed_dim = self.secondary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn on_secondary_fixed_variant(&self) -> Arc<DynamicColor> {
        let secondary_fixed_dim = self.secondary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tertiary_container = self.tertiary_container();
        let surface_bright = self.surface_bright();
        let surface_dim = self.surface_dim();
        Arc::new(DynamicColor::new(
            "tertiary_fixed".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            Some(Arc::new(move |scheme| {
                if scheme.platform == Platform::Phone {
                    if scheme.is_dark {
                        Some(surface_bright.clone())
                    } else {
                        Some(surface_dim.clone())
                    }
                } else {
                    None
                }
            })),
            Some(Arc::new(move |scheme| {
                let temp = DynamicScheme::from_scheme_with_contrast(scheme, false, 0.0);
                tertiary_container.get_tone(&temp)
            })),
            None,
            Some(Arc::new(|scheme| {
                if scheme.platform == Platform::Phone && scheme.contrast_level > 0.0 {
                    Some(Self::get_contrast_curve(1.5))
                } else {
                    None
                }
            })),
            None,
            None,
        ))
    }

    fn tertiary_fixed_dim(&self) -> Arc<DynamicColor> {
        let tertiary_fixed = self.tertiary_fixed();
        Arc::new(DynamicColor::new(
            "tertiary_fixed_dim".into(),
            Arc::new(|s| s.tertiary_palette.clone()),
            true,
            None,
            None,
            Some(Arc::new(move |scheme| tertiary_fixed.get_tone(scheme))),
            None,
            None,
            None,
            None,
        ))
    }

    fn on_tertiary_fixed(&self) -> Arc<DynamicColor> {
        let tertiary_fixed_dim = self.tertiary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn on_tertiary_fixed_variant(&self) -> Arc<DynamicColor> {
        let tertiary_fixed_dim = self.tertiary_fixed_dim();
        Arc::new(DynamicColor::new(
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
        ))
    }

    fn highest_surface(&self, scheme: &DynamicScheme) -> Arc<DynamicColor> {
        if scheme.platform == Platform::Phone {
            if scheme.is_dark {
                self.surface_bright()
            } else {
                self.surface_dim()
            }
        } else {
            self.surface_container_high()
        }
    }

    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct {
        let palette = (color.palette)(scheme);
        let tone = self.get_tone(scheme, color);
        let chroma = palette.chroma * color.chroma_multiplier.as_ref().map_or(1.0, |f| f(scheme));
        Hct::from(palette.hue, chroma, tone)
    }

    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64 {
        let tone_delta_pair = color.tone_delta_pair.as_ref().and_then(|f| f(scheme));
        if let Some(tdp) = tone_delta_pair {
            let absolute_delta = if tdp.polarity == TonePolarity::Darker
                || (tdp.polarity == TonePolarity::RelativeLighter && scheme.is_dark)
                || (tdp.polarity == TonePolarity::RelativeDarker && !scheme.is_dark)
            {
                -tdp.delta
            } else {
                tdp.delta
            };
            let am_role_a = color.name == tdp.role_a.name;
            let self_role = if am_role_a { &tdp.role_a } else { &tdp.role_b };
            let reference_role = if am_role_a { &tdp.role_b } else { &tdp.role_a };
            let mut self_tone = (self_role.tone)(scheme);
            let reference_tone = reference_role.get_tone(scheme);
            let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };
            match tdp.constraint {
                DeltaConstraint::Exact => {
                    self_tone = (reference_tone + relative_delta).clamp(0.0, 100.0)
                }
                DeltaConstraint::Nearer => {
                    if relative_delta > 0.0 {
                        self_tone = self_tone
                            .clamp(reference_tone, reference_tone + relative_delta)
                            .clamp(0.0, 100.0);
                    } else {
                        self_tone = self_tone
                            .clamp(reference_tone + relative_delta, reference_tone)
                            .clamp(0.0, 100.0);
                    }
                }
                DeltaConstraint::Farther => {
                    if relative_delta > 0.0 {
                        self_tone = self_tone.clamp(reference_tone + relative_delta, 100.0);
                    } else {
                        self_tone = self_tone.clamp(0.0, reference_tone + relative_delta);
                    }
                }
            }
            if let (Some(bg), Some(curve)) = (
                color.background.as_ref().and_then(|f| f(scheme)),
                color.contrast_curve.as_ref().and_then(|f| f(scheme)),
            ) {
                let bg_tone = bg.get_tone(scheme);
                let self_contrast = curve.get(scheme.contrast_level);
                if Contrast::ratio_of_tones(bg_tone, self_tone) < self_contrast
                    || scheme.contrast_level < 0.0
                {
                    self_tone = DynamicColor::foreground_tone(bg_tone, self_contrast);
                }
            }
            if color.is_background && !color.name.ends_with("_fixed_dim") {
                self_tone = if self_tone >= 57.0 {
                    self_tone.clamp(65.0, 100.0)
                } else {
                    self_tone.clamp(0.0, 49.0)
                };
            }
            return self_tone;
        }

        let mut answer = (color.tone)(scheme);
        let background = color.background.as_ref().and_then(|f| f(scheme));
        let contrast_curve = color.contrast_curve.as_ref().and_then(|f| f(scheme));
        let (Some(background), Some(contrast_curve)) = (background, contrast_curve) else {
            return answer;
        };
        let bg_tone = background.get_tone(scheme);
        let desired_ratio = contrast_curve.get(scheme.contrast_level);
        if Contrast::ratio_of_tones(bg_tone, answer) < desired_ratio || scheme.contrast_level < 0.0
        {
            answer = DynamicColor::foreground_tone(bg_tone, desired_ratio);
        }
        if color.is_background && !color.name.ends_with("_fixed_dim") {
            answer = if answer >= 57.0 {
                answer.clamp(65.0, 100.0)
            } else {
                answer.clamp(0.0, 49.0)
            };
        }
        let second_background = color.second_background.as_ref().and_then(|f| f(scheme));
        let Some(second_background) = second_background else {
            return answer;
        };
        let bg_tone1 = background.get_tone(scheme);
        let bg_tone2 = second_background.get_tone(scheme);
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
            (Some(v), None) => v,
            (Some(_), Some(d)) => d,
            (None, Some(d)) => d,
            (None, None) => 0.0,
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
                } else if Hct::is_blue(source_color_hct.hue()) {
                    16.0
                } else {
                    12.0
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
                } else if Hct::is_blue(source_color_hct.hue()) {
                    10.0
                } else {
                    6.0
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
            Variant::Expressive => TonalPalette::from_hue_and_chroma(
                Self::get_expressive_neutral_hue(source_color_hct),
                Self::get_expressive_neutral_chroma(source_color_hct, is_dark, platform),
            ),
            Variant::Vibrant => TonalPalette::from_hue_and_chroma(
                Self::get_vibrant_neutral_hue(source_color_hct),
                Self::get_vibrant_neutral_chroma(source_color_hct, platform),
            ),
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
                let hue = Self::get_expressive_neutral_hue(source_color_hct);
                let chroma =
                    Self::get_expressive_neutral_chroma(source_color_hct, is_dark, platform);
                TonalPalette::from_hue_and_chroma(
                    hue,
                    chroma
                        * if (105.0..125.0).contains(&hue) {
                            1.6
                        } else {
                            2.3
                        },
                )
            }
            Variant::Vibrant => {
                let hue = Self::get_vibrant_neutral_hue(source_color_hct);
                let chroma = Self::get_vibrant_neutral_chroma(source_color_hct, platform);
                TonalPalette::from_hue_and_chroma(hue, chroma * 1.29)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::color_spec::SpecVersion;
    use crate::utils::color_utils::Argb;

    fn make_scheme(variant: Variant, is_dark: bool, platform: Platform) -> DynamicScheme {
        let source = Hct::from_int(Argb(0xFF4285F4));
        let spec = ColorSpec2025::new();
        DynamicScheme::new_with_platform_and_spec(
            source,
            variant,
            is_dark,
            0.0,
            platform,
            SpecVersion::Spec2025,
            spec.get_primary_palette(variant, &source, is_dark, platform, 0.0),
            spec.get_secondary_palette(variant, &source, is_dark, platform, 0.0),
            spec.get_tertiary_palette(variant, &source, is_dark, platform, 0.0),
            spec.get_neutral_palette(variant, &source, is_dark, platform, 0.0),
            spec.get_neutral_variant_palette(variant, &source, is_dark, platform, 0.0),
            spec.get_error_palette(variant, &source, is_dark, platform, 0.0),
        )
    }

    #[test]
    fn background_tracks_surface_in_2025() {
        let spec = ColorSpec2025::new();
        let scheme = make_scheme(Variant::TonalSpot, false, Platform::Phone);
        assert_eq!(
            spec.background().get_tone(&scheme),
            spec.surface().get_tone(&scheme)
        );
    }

    #[test]
    fn surface_variant_tracks_surface_container_highest_in_2025() {
        let spec = ColorSpec2025::new();
        let scheme = make_scheme(Variant::Expressive, false, Platform::Phone);
        assert_eq!(
            spec.surface_variant().get_tone(&scheme),
            spec.surface_container_highest().get_tone(&scheme)
        );
    }

    #[test]
    fn primary_dim_exists_in_2025() {
        let spec = ColorSpec2025::new();
        assert!(spec.primary_dim().is_some());
    }

    #[test]
    fn surface_watch_tone_is_zero() {
        let spec = ColorSpec2025::new();
        let scheme = make_scheme(Variant::Neutral, false, Platform::Watch);
        assert_eq!(spec.surface().get_tone(&scheme), 0.0);
    }
}
