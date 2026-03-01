use crate::dynamic::color_spec::{ColorSpec, SpecVersion};
use crate::dynamic::color_spec_2021::ColorSpec2021;
use crate::dynamic::color_spec_2025::ColorSpec2025;
use crate::dynamic::color_spec_2026::ColorSpec2026;
use bon::bon;
use std::sync::OnceLock;

/// A utility struct to get the correct color spec for a given spec version.
pub struct ColorSpecs;

static SPEC_2021: OnceLock<ColorSpec2021> = OnceLock::new();
static SPEC_2025: OnceLock<ColorSpec2025> = OnceLock::new();
static SPEC_2026: OnceLock<ColorSpec2026> = OnceLock::new();

#[bon]
impl ColorSpecs {
    /// Gets the default `ColorSpec` (Spec2021).
    #[must_use]
    pub fn get_default() -> &'static dyn ColorSpec {
        Self::get(SpecVersion::Spec2021).call()
    }

    /// Returns a builder to get a `ColorSpec` for the requested `spec_version`.
    ///
    /// The `spec_version` is passed as a positional argument to `.get()`.
    /// The fidelity setting is optional and defaults to `false`.
    #[builder(start_fn = get)]
    #[must_use]
    pub fn get_impl(
        #[builder(start_fn)] spec_version: SpecVersion,
        /// Whether to use extended fidelity settings. Defaults to `false`.
        #[builder(default = false)]
        _is_extended_fidelity: bool,
    ) -> &'static dyn ColorSpec {
        match spec_version {
            SpecVersion::Spec2021 => SPEC_2021.get_or_init(ColorSpec2021::new),
            SpecVersion::Spec2025 => SPEC_2025.get_or_init(ColorSpec2025::new),
            SpecVersion::Spec2026 => SPEC_2026.get_or_init(ColorSpec2026::new),
        }
    }
}
