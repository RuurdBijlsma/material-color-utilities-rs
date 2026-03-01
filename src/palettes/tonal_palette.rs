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

use crate::hct::hct_color::Hct;
use crate::utils::color_utils::Argb;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// A convenience class for retrieving colors that are constant in hue and chroma, but vary in tone.
#[derive(Debug)]
pub struct TonalPalette {
    /// The hue of the Tonal Palette, in HCT. Ranges from 0 to 360.
    pub hue: f64,
    /// The chroma of the Tonal Palette, in HCT. Ranges from 0 to ~130 (for sRGB gamut).
    pub chroma: f64,
    /// The key color is the first tone, starting from T50, that matches the palette's chroma.
    pub key_color: Hct,
    /// Cache that maps tone to ARGB color to avoid duplicated HCT calculation.
    cache: Arc<[AtomicU32; 101]>,
}

impl Clone for TonalPalette {
    fn clone(&self) -> Self {
        Self {
            hue: self.hue,
            chroma: self.chroma,
            key_color: self.key_color,
            cache: self.cache.clone(),
        }
    }
}

impl PartialEq for TonalPalette {
    fn eq(&self, other: &Self) -> bool {
        self.key_color == other.key_color
    }
}

impl Eq for TonalPalette {}

impl std::hash::Hash for TonalPalette {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key_color.hash(state);
    }
}

impl TonalPalette {
    fn new(hue: f64, chroma: f64, key_color: Hct) -> Self {
        Self {
            hue,
            chroma,
            key_color,
            cache: Arc::new(std::array::from_fn(|_| AtomicU32::new(0))),
        }
    }

    /// Create tones using the HCT hue and chroma from a color.
    ///
    /// # Arguments
    ///
    /// * `argb`: ARGB representation of a color
    ///
    /// # Returns
    ///
    /// `TonalPalette` matching that color's hue and chroma.
    #[must_use]
    pub fn from_argb(argb: Argb) -> Self {
        Self::from_hct(Hct::from_argb(argb))
    }

    /// Create tones using a HCT color.
    ///
    /// # Arguments
    ///
    /// * `hct`: HCT representation of a color.
    ///
    /// # Returns
    ///
    /// `TonalPalette` matching that color's hue and chroma.
    #[must_use]
    pub fn from_hct(hct: Hct) -> Self {
        Self::new(hct.hue(), hct.chroma(), hct)
    }

    /// Create tones from a defined HCT hue and chroma.
    ///
    /// # Arguments
    ///
    /// * `hue`: HCT hue
    /// * `chroma`: HCT chroma
    ///
    /// # Returns
    ///
    /// `TonalPalette` matching hue and chroma.
    #[must_use]
    pub fn from_hue_and_chroma(hue: f64, chroma: f64) -> Self {
        let key_color = KeyColor::new(hue, chroma).create();
        Self::new(hue, chroma, key_color)
    }

    /// Create an ARGB color with HCT hue and chroma of this Tones instance, and the provided HCT tone.
    ///
    /// # Arguments
    ///
    /// * `tone`: HCT tone, measured from 0 to 100.
    ///
    /// # Returns
    ///
    /// ARGB representation of a color with that tone.
    #[must_use] 
    pub fn tone(&self, tone: i32) -> Argb {
        if !(0..=100).contains(&tone) {
            return Hct::from(self.hue, self.chroma, f64::from(tone)).to_argb();
        }

        let index = tone as usize;
        let cached = self.cache[index].load(Ordering::Relaxed);
        if cached != 0 {
            return Argb(cached);
        }

        let color = if tone == 99 && Hct::is_yellow(self.hue) {
            Self::average_argb(self.tone(98), self.tone(100))
        } else {
            Hct::from(self.hue, self.chroma, f64::from(tone)).to_argb()
        };

        self.cache[index].store(color.0, Ordering::Relaxed);
        color
    }

    /// Given a tone, use hue and chroma of palette to create a color, and return it as HCT.
    #[must_use] 
    pub fn get_hct(&self, tone: f64) -> Hct {
        Hct::from(self.hue, self.chroma, tone)
    }

    fn average_argb(argb1: Argb, argb2: Argb) -> Argb {
        let red1 = f32::from(argb1.red());
        let green1 = f32::from(argb1.green());
        let blue1 = f32::from(argb1.blue());
        let red2 = f32::from(argb2.red());
        let green2 = f32::from(argb2.green());
        let blue2 = f32::from(argb2.blue());

        let red = f32::midpoint(red1, red2).round() as u8;
        let green = f32::midpoint(green1, green2).round() as u8;
        let blue = f32::midpoint(blue1, blue2).round() as u8;

        Argb::from_rgb(red, green, blue)
    }
}

/// Key color is a color that represents the hue and chroma of a tonal palette.
struct KeyColor {
    hue: f64,
    requested_chroma: f64,
}

impl KeyColor {
    const MAX_CHROMA_VALUE: f64 = 200.0;

    const fn new(hue: f64, requested_chroma: f64) -> Self {
        Self {
            hue,
            requested_chroma,
        }
    }

    /// Creates a key color from a [hue] and a [chroma]. The key color is the first tone, starting
    /// from T50, matching the given hue and chroma.
    ///
    /// @return Key color [Hct]
    fn create(&self) -> Hct {
        // Pivot around T50 because T50 has the most chroma available, on
        // average. Thus it is most likely to have a direct answer.
        let pivot_tone = 50;
        let tone_step_size = 1;
        // Epsilon to accept values slightly higher than the requested chroma.
        let epsilon = 0.01;

        // Binary search to find the tone that can provide a chroma that is closest
        // to the requested chroma.
        let mut lower_tone = 0;
        let mut upper_tone = 100;
        while lower_tone < upper_tone {
            let mid_tone = i32::midpoint(lower_tone, upper_tone);
            let is_ascending =
                self.max_chroma(mid_tone) < self.max_chroma(mid_tone + tone_step_size);
            let sufficient_chroma = self.max_chroma(mid_tone) >= self.requested_chroma - epsilon;
            if sufficient_chroma {
                // Either range [lowerTone, midTone] or [midTone, upperTone] has
                // the answer, so search in the range that is closer the pivot tone.
                if (lower_tone - pivot_tone).abs() < (upper_tone - pivot_tone).abs() {
                    upper_tone = mid_tone;
                } else {
                    if lower_tone == mid_tone {
                        return Hct::from(self.hue, self.requested_chroma, f64::from(lower_tone));
                    }
                    lower_tone = mid_tone;
                }
            } else {
                // As there is no sufficient chroma in the midTone, follow the direction to the chroma
                // peak.
                if is_ascending {
                    lower_tone = mid_tone + tone_step_size;
                } else {
                    // Keep midTone for potential chroma peak.
                    upper_tone = mid_tone;
                }
            }
        }
        Hct::from(self.hue, self.requested_chroma, f64::from(lower_tone))
    }

    // Find the maximum chroma for a given tone
    fn max_chroma(&self, tone: i32) -> f64 {
        Hct::from(self.hue, Self::MAX_CHROMA_VALUE, f64::from(tone)).chroma()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tonal_palette_from_argb() {
        let argb = Argb(0xFF0000FF); // Blue
        let palette = TonalPalette::from_argb(argb);
        assert!((palette.hue - 282.12).abs() < 1.0); // Approximate hue for pure blue
        assert!(palette.chroma > 80.0);
    }

    #[test]
    fn test_tone_caching() {
        let palette = TonalPalette::from_hue_and_chroma(120.0, 40.0);
        let color1 = palette.tone(50);
        let color2 = palette.tone(50);
        assert_eq!(color1, color2);

        let cached = palette.cache[50].load(Ordering::Relaxed);
        assert_eq!(cached, color1.0);
    }

    #[test]
    fn test_yellow_tone_99() {
        // Hue 110 is in the yellow range (105-125)
        let palette = TonalPalette::from_hue_and_chroma(110.0, 40.0);
        let tone99 = palette.tone(99);
        let tone98 = palette.tone(98);
        let tone100 = palette.tone(100);

        // tone 99 should be average of 98 and 100
        let red = f32::midpoint(f32::from(tone98.red()), f32::from(tone100.red())).round() as u8;
        let green =
            f32::midpoint(f32::from(tone98.green()), f32::from(tone100.green())).round() as u8;
        let blue = f32::midpoint(f32::from(tone98.blue()), f32::from(tone100.blue())).round() as u8;
        let expected = Argb::from_rgb(red, green, blue);

        assert_eq!(tone99, expected);
    }

    #[test]
    fn test_key_color() {
        let hue = 200.0;
        let chroma = 30.0;
        let palette = TonalPalette::from_hue_and_chroma(hue, chroma);

        // Key color should have the requested hue and roughly the requested chroma
        assert!((palette.key_color.hue() - hue).abs() < 1.0);
        assert!((palette.key_color.chroma() - chroma).abs() < 1.0);
    }

    #[test]
    fn test_out_of_bounds_tone() {
        let palette = TonalPalette::from_hue_and_chroma(120.0, 40.0);
        // Should not panic and should return a color
        let color = palette.tone(150);
        assert_ne!(color.0, 0);

        let color_neg = palette.tone(-10);
        assert_ne!(color_neg.0, 0);
    }
}
