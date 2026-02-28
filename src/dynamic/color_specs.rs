use crate::dynamic::color_spec::{ColorSpec, SpecVersion};
use crate::dynamic::color_spec_2021::ColorSpec2021;
use crate::dynamic::color_spec_2025::ColorSpec2025;
use crate::dynamic::color_spec_2026::ColorSpec2026;
use std::sync::OnceLock;

/// A utility struct to get the correct color spec for a given spec version.
pub struct ColorSpecs;

static SPEC_2021: OnceLock<ColorSpec2021> = OnceLock::new();
static SPEC_2025: OnceLock<ColorSpec2025> = OnceLock::new();
static SPEC_2026: OnceLock<ColorSpec2026> = OnceLock::new();

impl ColorSpecs {
    /// Gets the default `ColorSpec` (Spec2021).
    #[must_use]
    pub fn get_default() -> &'static dyn ColorSpec {
        Self::get(SpecVersion::Spec2021)
    }

    /// Return a boxed `ColorSpec` for the requested `spec_version`.
    #[must_use]
    pub fn get(spec_version: SpecVersion) -> &'static dyn ColorSpec {
        Self::get_with_fidelity(spec_version, false)
    }

    /// Return a boxed `ColorSpec` for the requested `spec_version` and fidelity setting.
    #[must_use]
    pub fn get_with_fidelity(
        spec_version: SpecVersion,
        _is_extended_fidelity: bool,
    ) -> &'static dyn ColorSpec {
        match spec_version {
            SpecVersion::Spec2021 => SPEC_2021.get_or_init(ColorSpec2021::new),
            SpecVersion::Spec2025 => SPEC_2025.get_or_init(ColorSpec2025::new),
            SpecVersion::Spec2026 => SPEC_2026.get_or_init(ColorSpec2026::new),
        }
    }
}
