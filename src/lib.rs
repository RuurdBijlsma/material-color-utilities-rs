//! # material-color-utils
//!
//! A high-performance Rust port of [Material Color Utilities](https://github.com/material-foundation/material-color-utilities).
//!
//! This crate provides algorithms and utilities for working with Material 3 (M3) dynamic color,
//! including the HCT color space, color quantization, and scheme generation.
//!
//! ## Core Concepts
//!
//! *   **HCT (Hue, Chroma, Tone):** A perceptually accurate color space that separates hue from lightness (tone).
//! *   **Dynamic Color:** A system that generates adaptive, accessible color schemes based on a user's source color.
//! *   **Materialized Theme:** A flattened representation of a color scheme, easy to use in UI frameworks.
//!
//! ## Usage Examples
//!
//! ### High-Level Helpers (Simplified Theme Generation)
//!
//! The easiest way to get started is with the high-level `theme_from_color` helper.
//!
//! ```rust
//! use material_color_utils::utils::color_utils::Argb;
//! use material_color_utils::theme_from_color;
//!
//! // Create a theme from a source ARGB color
//! let source_color = Argb::from_hex("#4285F4").unwrap(); // Blue
//! // Builder with optional arguments for contrast, scheme variant, and more:
//! let theme = theme_from_color(source_color).call();
//!
//! // Access light and dark schemes
//! let primary_light = theme.schemes.light.primary;
//! let primary_dark = theme.schemes.dark.primary;
//!
//! println!("Light Primary: {}", primary_light);
//! println!("Dark Primary: {}", primary_dark);
//!
//! // The materialized theme structs support serde (de)serialization
//! println!("Theme json: {}", serde_json::to_string_pretty(&theme).unwrap())
//! ```
//!
//! ### Color Extraction from Image
//!
//! The `image` feature enables extracting prominent colors from images to use as source colors.
//!
//! ```rust
//! use material_color_utils::{extract_image_colors, theme_from_image};
//!
//! let img = image::open("tests/assets/img/river.png").unwrap();
//!
//! // Extract colors from an image
//! let colors = extract_image_colors(&img).call();
//! println!("Colors in 'river.png' to make a theme from: {:?}", colors);
//!
//! // Generate a theme directly from an image
//! let theme = theme_from_image(&img).call().unwrap();
//! ```
//!
//! ### Dynamic API (HCT & Custom Schemes)
//!
//! For more control, and lazy evaluation,
//! you can work directly with the HCT color space and individual scheme builders.
//!
//! ```rust
//! use material_color_utils::hct::Hct;
//! use material_color_utils::scheme::SchemeTonalSpot;
//! use material_color_utils::dynamic::material_dynamic_colors::MaterialDynamicColors;
//! use material_color_utils::dynamic::color_spec::SpecVersion;
//! use material_color_utils::utils::color_utils::Argb;
//!
//! // 1. Create a color in HCT space
//! let hct = Hct::from_argb(Argb(0xFF4285F4));
//! println!("H: {}, C: {}, T: {}", hct.hue(), hct.chroma(), hct.tone());
//!
//! // 2. Manually build a scheme (Tonal Spot, Dark Mode, High Contrast)
//! let scheme = SchemeTonalSpot::builder(hct.to_argb(), true, 0.5)
//!     .spec_version(SpecVersion::Spec2026)
//!     .build();
//!
//! // 3. Extract specific color roles using MaterialDynamicColors
//! let mdc = MaterialDynamicColors::new();
//! let primary = mdc.primary().get_argb(&scheme);
//! let on_primary = mdc.on_primary().get_argb(&scheme);
//!
//! println!("Primary: {:?}", primary);
//! println!("On-Primary: {:?}", on_primary);
//! ```
//!
//! ### Contrast Helpers
//!
//! Utilities for calculating contrast ratios and adjusting colors to meet accessibility standards.
//!
//! ```rust
//! use material_color_utils::utils::color_utils::Argb;
//! use material_color_utils::{get_contrast_ratio, lighter_tone, darker_tone, lighter_tone_unsafe, darker_tone_unsafe};
//!
//! let color1 = Argb(0xFF4285F4);
//! let color2 = Argb::from_hex("#FFFFFF").unwrap();
//!
//! // Calculate contrast ratio
//! let ratio = get_contrast_ratio(color1, color2);
//! println!("Contrast ratio: {:.2}", ratio);
//!
//! // Find a color that meets a target contrast ratio
//! if let Some(lighter) = lighter_tone(color1, 4.5) {
//!     println!("Lighter color with 4.5 contrast: {}", lighter);
//! }
//!
//! // lighter_tone_unsafe will clip to white if it can't reach the desired contrast ratio.
//! let lighter_unsafe = lighter_tone_unsafe(color1, 4.5);
//! let darker_unsafe = darker_tone_unsafe(color1, 4.5);
//! println!("Lighter color, clipped if necessary at tone=100 (white): {}", lighter_unsafe);
//! println!("Darker color, clipped if necessary at tone=0 (black): {}", lighter_unsafe);
//! ```
//!
//! ### UI Integration
//!
//! Dynamic colors are designed for lazy evaluation, allowing for high-performance,
//! interfaces where only the colors used in the UI are calculated.
//!
//! ```rust
//! use material_color_utils::dynamic::dynamic_scheme::DynamicScheme;
//! use material_color_utils::dynamic::material_dynamic_colors::MaterialDynamicColors;
//!
//! struct MyUIComponent {
//!     scheme: DynamicScheme,
//!     mdc: MaterialDynamicColors,
//! }
//!
//! impl MyUIComponent {
//!     fn on_render(&self) {
//!         // Colors are evaluated on-demand from the current scheme
//!         let bg = self.mdc.surface().get_argb(&self.scheme);
//!         let primary = self.mdc.primary().get_argb(&self.scheme);
//!
//!         // ... apply bg and primary to UI elements ...
//!     }
//! }
//! ```

#![deny(clippy::unwrap_used)]
#![allow(
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::many_single_char_names,
    clippy::while_float,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::match_wildcard_for_single_variants,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_long_first_doc_paragraph,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
pub mod blend;
pub mod contrast;
pub mod dislike;
pub mod dynamic;
pub mod hct;
mod helpers;
pub mod palettes;
pub mod quantize;
pub mod scheme;
pub mod score;
pub mod temperature;
pub mod utils;

pub use helpers::*;
