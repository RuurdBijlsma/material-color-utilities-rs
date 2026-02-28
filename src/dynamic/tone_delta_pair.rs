use crate::dynamic::dynamic_color::DynamicColor;
use std::fmt::Debug;
use std::sync::Arc;

/// Describes how to fulfill a tone delta pair constraint.
///
/// Determines if the delta is a minimum, maximum, or exact tonal distance that must be maintained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaConstraint {
    /// The tone of roleA must be an exact delta away from the tone of roleB.
    Exact,
    /// The tonal distance of roleA and roleB must be at most delta.
    Nearer,
    /// The tonal distance of roleA and roleB must be at least delta.
    Farther,
}

/// Describes the relationship in lightness between two colors.
///
/// '`relative_darker`' and '`relative_lighter`' describes the tone adjustment relative to the surface
/// color trend (white in light mode; black in dark mode). For instance, ToneDeltaPair(A, B, 10,
/// '`relative_lighter`', 'farther') states that A should be at least 10 lighter than B in light
/// mode, and at least 10 darker than B in dark mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonePolarity {
    /// The tone of roleA is always darker than the tone of roleB.
    Darker,
    /// The tone of roleA is always lighter than the tone of roleB.
    Lighter,
    /// The tone of roleA is darker than the tone of roleB in light mode, and lighter than the tone
    /// of roleB in dark mode.
    RelativeDarker,
    /// The tone of roleA is lighter than the tone of roleB in light mode, and darker than the tone
    /// of roleB in dark mode.
    RelativeLighter,
}

/// Documents a constraint between two `DynamicColors`, in which their tones must have a certain
/// distance from each other.
///
/// The polarity is an adjective that describes "A", compared to "B". For instance, ToneDeltaPair(A,
/// B, 15, 'darker') states that A's tone should be at least 15 darker than B's.
///
/// Prefer a `DynamicColor` with a background, this is for special cases when designers want tonal
/// distance, literally contrast, between two colors that don't have a background / foreground
/// relationship or a contrast guarantee.
#[derive(Clone, Debug)]
pub struct ToneDeltaPair {
    pub role_a: Arc<DynamicColor>,
    pub role_b: Arc<DynamicColor>,
    pub delta: f64,
    pub polarity: TonePolarity,
    pub stay_together: bool,
    pub constraint: DeltaConstraint,
}

impl ToneDeltaPair {
    #[must_use]
    pub const fn new(
        role_a: Arc<DynamicColor>,
        role_b: Arc<DynamicColor>,
        delta: f64,
        polarity: TonePolarity,
        stay_together: bool,
        constraint: DeltaConstraint,
    ) -> Self {
        Self {
            role_a,
            role_b,
            delta,
            polarity,
            stay_together,
            constraint,
        }
    }
}
