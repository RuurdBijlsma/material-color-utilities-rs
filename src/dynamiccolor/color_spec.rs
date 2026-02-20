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
use crate::dynamiccolor::color_spec_2021::ColorSpec2021;
use crate::hct::hct::Hct;
use crate::dynamiccolor::dynamic_color::DynamicColor;

/// All available spec versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecVersion {
    Spec2021,
    Spec2025,
    Spec2026,
}

/// An interface defining all the necessary methods that could be different between specs.
pub trait ColorSpec {
    fn get_hct(&self, scheme: &DynamicScheme, color: &DynamicColor) -> Hct;
    fn get_tone(&self, scheme: &DynamicScheme, color: &DynamicColor) -> f64;
}

// Placeholder for DynamicScheme (defined properly in its own file)
use crate::dynamiccolor::dynamic_scheme::DynamicScheme;

/// A utility to get the correct color spec for a given spec version.
pub struct ColorSpecs;

impl ColorSpecs {
    pub fn get(spec_version: SpecVersion) -> Box<dyn ColorSpec> {
        match spec_version {
            _ => Box::new(ColorSpec2021::new()),
        }
    }
}