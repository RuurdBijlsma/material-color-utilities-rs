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

use crate::dynamiccolor::color_spec::{ColorSpec, SpecVersion};
use crate::dynamiccolor::color_spec_2021::ColorSpec2021;
use crate::dynamiccolor::color_spec_2025::ColorSpec2025;
use crate::dynamiccolor::color_spec_2026::ColorSpec2026;

/// A utility struct to get the correct color spec for a given spec version.
pub struct ColorSpecs;

impl ColorSpecs {
    /// Gets the default `ColorSpec` (Spec2021).
    #[must_use]
    pub fn get_default() -> Box<dyn ColorSpec> {
        Self::get(SpecVersion::Spec2021)
    }

    /// Return a boxed `ColorSpec` for the requested `spec_version`.
    #[must_use]
    pub fn get(spec_version: SpecVersion) -> Box<dyn ColorSpec> {
        Self::get_with_fidelity(spec_version, false)
    }

    /// Return a boxed `ColorSpec` for the requested `spec_version` and fidelity setting.
    #[must_use]
    pub fn get_with_fidelity(
        spec_version: SpecVersion,
        _is_extended_fidelity: bool,
    ) -> Box<dyn ColorSpec> {
        match spec_version {
            SpecVersion::Spec2025 => Box::new(ColorSpec2025::new()),
            SpecVersion::Spec2026 => Box::new(ColorSpec2026::new()),
            _ => Box::new(ColorSpec2021::new()),
        }
    }
}
