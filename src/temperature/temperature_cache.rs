/*
 * Copyright 2022 Google LLC
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

use crate::hct::hct::Hct;
use crate::utils::color_utils::Argb;
use crate::utils::math_utils::MathUtils;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Design utilities using color temperature theory.
///
/// Analogous colors, complementary color, and cache to efficiently, lazily, generate data for
/// calculations when needed.
pub struct TemperatureCache {
    input: Hct,
    precomputed_complement: OnceLock<Hct>,
    precomputed_hcts_by_temp: OnceLock<Vec<Hct>>,
    precomputed_hcts_by_hue: OnceLock<Vec<Hct>>,
    precomputed_temps_by_hct: OnceLock<HashMap<Argb, f64>>,
}

impl TemperatureCache {
    pub fn new(input: Hct) -> Self {
        Self {
            input,
            precomputed_complement: OnceLock::new(),
            precomputed_hcts_by_temp: OnceLock::new(),
            precomputed_hcts_by_hue: OnceLock::new(),
            precomputed_temps_by_hct: OnceLock::new(),
        }
    }

    /// A color that complements the input color aesthetically.
    ///
    /// In art, this is usually described as being across the color wheel. History of this shows intent
    /// as a color that is just as cool-warm as the input color is warm-cool.
    pub fn complement(&self) -> Hct {
        *self.precomputed_complement.get_or_init(|| {
            let coldest = self.coldest();
            let temps = self.temps_by_hct();
            
            let coldest_hue = coldest.hue();
            let coldest_temp = *temps.get(&coldest.to_int()).unwrap();
            
            let warmest = self.warmest();
            let warmest_hue = warmest.hue();
            let warmest_temp = *temps.get(&warmest.to_int()).unwrap();
            
            let range = warmest_temp - coldest_temp;
            let start_hue_is_coldest_to_warmest = Self::is_between(self.input.hue(), coldest_hue, warmest_hue);
            
            let start_hue = if start_hue_is_coldest_to_warmest { warmest_hue } else { coldest_hue };
            let end_hue = if start_hue_is_coldest_to_warmest { coldest_hue } else { warmest_hue };
            
            let direction_of_rotation = 1.0;
            let mut smallest_error = 1000.0;
            
            let hcts_by_hue = self.hcts_by_hue();
            let mut answer = hcts_by_hue[self.input.hue().round() as usize % 360];
            
            let complement_relative_temp = 1.0 - self.get_relative_temperature(&self.input);
            
            // Find the color in the other section, closest to the inverse percentile
            // of the input color. This is the complement.
            let mut hue_addend = 0.0;
            while hue_addend <= 360.0 {
                let hue = MathUtils::sanitize_degrees_double(start_hue + direction_of_rotation * hue_addend);
                if !Self::is_between(hue, start_hue, end_hue) {
                    hue_addend += 1.0;
                    continue;
                }
                
                let possible_answer = hcts_by_hue[hue.round() as usize % 360];
                let relative_temp = (*temps.get(&possible_answer.to_int()).unwrap() - coldest_temp) / range;
                let error = (complement_relative_temp - relative_temp).abs();
                if error < smallest_error {
                    smallest_error = error;
                    answer = possible_answer;
                }
                hue_addend += 1.0;
            }
            answer
        })
    }

    /// 5 colors that pair well with the input color.
    ///
    /// The colors are equidistant in temperature and adjacent in hue.
    pub fn get_analogous_colors(&self) -> Vec<Hct> {
        self.get_analogous_colors_with_options(5, 12)
    }

    /// A set of colors with differing hues, equidistant in temperature.
    ///
    /// In art, this is usually described as a set of 5 colors on a color wheel divided into 12
    /// sections. This method allows provision of either of those values.
    ///
    /// Behavior is undefined when count or divisions is 0. When divisions < count, colors repeat.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of colors to return, includes the input color.
    /// * `divisions` - The number of divisions on the color wheel.
    pub fn get_analogous_colors_with_options(&self, count: usize, divisions: usize) -> Vec<Hct> {
        // The starting hue is the hue of the input color.
        let start_hue = self.input.hue().round() as i32;
        let hcts_by_hue = self.hcts_by_hue();
        let start_hct = hcts_by_hue[MathUtils::sanitize_degrees_int(start_hue) as usize % 360];
        let mut last_temp = self.get_relative_temperature(&start_hct);
        
        let mut all_colors: Vec<Hct> = Vec::new();
        all_colors.push(start_hct);
        
        let mut absolute_total_temp_delta = 0.0;
        for i in 0..360 {
            let hue = MathUtils::sanitize_degrees_int(start_hue + i);
            let hct = hcts_by_hue[hue as usize % 360];
            let temp = self.get_relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            last_temp = temp;
            absolute_total_temp_delta += temp_delta;
        }
        
        let mut hue_addend = 1;
        let temp_step = absolute_total_temp_delta / divisions as f64;
        let mut total_temp_delta = 0.0;
        last_temp = self.get_relative_temperature(&start_hct);
        
        while all_colors.len() < divisions {
            let hue = MathUtils::sanitize_degrees_int(start_hue + hue_addend);
            let hct = hcts_by_hue[hue as usize % 360];
            let temp = self.get_relative_temperature(&hct);
            let temp_delta = (temp - last_temp).abs();
            total_temp_delta += temp_delta;
            
            let mut desired_total_temp_delta_for_index = all_colors.len() as f64 * temp_step;
            let mut index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
            let mut index_addend = 1;
            
            // Keep adding this hue to the answers until its temperature is
            // insufficient. This ensures consistent behavior when there aren't
            // `divisions` discrete steps between 0 and 360 in hue with `temp_step`
            // delta in temperature between them.
            while index_satisfied && all_colors.len() < divisions {
                all_colors.push(hct);
                desired_total_temp_delta_for_index = (all_colors.len() + index_addend) as f64 * temp_step;
                index_satisfied = total_temp_delta >= desired_total_temp_delta_for_index;
                index_addend += 1;
            }
            last_temp = temp;
            hue_addend += 1;
            
            if hue_addend > 360 {
                while all_colors.len() < divisions {
                    all_colors.push(hct);
                }
                break;
            }
        }
        
        let mut answers: Vec<Hct> = Vec::new();
        answers.push(self.input);
        
        let ccw_count = ((count as f64 - 1.0) / 2.0).floor() as usize;
        for i in 1..=ccw_count {
            let mut index = 0i32 - i as i32;
            while index < 0 {
                index += all_colors.len() as i32;
            }
            let idx = (index as usize) % all_colors.len();
            answers.insert(0, all_colors[idx]);
        }
        
        let cw_count = count - ccw_count - 1;
        for i in 1..=cw_count {
            let index = i;
            let idx = index % all_colors.len();
            answers.push(all_colors[idx]);
        }
        
        answers
    }

    /// Temperature relative to all colors with the same chroma and tone.
    ///
    /// @param hct HCT to find the relative temperature of.
    /// @return Value on a scale from 0 to 1.
    pub fn get_relative_temperature(&self, hct: &Hct) -> f64 {
        let temps = self.temps_by_hct();
        let coldest_temp = *temps.get(&self.coldest().to_int()).unwrap();
        let warmest_temp = *temps.get(&self.warmest().to_int()).unwrap();
        
        let range = warmest_temp - coldest_temp;
        let difference_from_coldest = *temps.get(&hct.to_int()).unwrap() - coldest_temp;
        
        // Handle when there's no difference in temperature between warmest and
        // coldest: for example, at T100, only one color is available, white.
        if range == 0.0 {
            0.5
        } else {
            difference_from_coldest / range
        }
    }

    /// Coldest color with same chroma and tone as input.
    fn coldest(&self) -> Hct {
        self.hcts_by_temp()[0]
    }

    /// Warmest color with same chroma and tone as input.
    fn warmest(&self) -> Hct {
        let hcts = self.hcts_by_temp();
        hcts[hcts.len() - 1]
    }

    /// HCTs for all colors with the same chroma/tone as the input.
    ///
    /// Sorted by hue, ex. index 0 is hue 0.
    fn hcts_by_hue(&self) -> &[Hct] {
        self.precomputed_hcts_by_hue.get_or_init(|| {
            let mut hcts = Vec::new();
            let mut hue = 0.0;
            while hue < 360.0 {
                let color_at_hue = Hct::from(hue, self.input.chroma(), self.input.tone());
                hcts.push(color_at_hue);
                hue += 1.0;
            }
            hcts
        })
    }

    /// HCTs for all colors with the same chroma/tone as the input.
    ///
    /// Sorted from coldest first to warmest last.
    fn hcts_by_temp(&self) -> &[Hct] {
        self.precomputed_hcts_by_temp.get_or_init(|| {
            let mut hcts = self.hcts_by_hue().to_vec();
            hcts.push(self.input);
            let temps = self.temps_by_hct();
            hcts.sort_by(|a, b| {
                let temp_a = temps.get(&a.to_int()).unwrap();
                let temp_b = temps.get(&b.to_int()).unwrap();
                temp_a.partial_cmp(temp_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            hcts
        })
    }

    /// Keys of HCTs in getHctsByTemp, values of raw temperature.
    fn temps_by_hct(&self) -> &HashMap<Argb, f64> {
        self.precomputed_temps_by_hct.get_or_init(|| {
            let mut all_hcts = self.hcts_by_hue().to_vec();
            all_hcts.push(self.input);
            
            let mut temperatures_by_hct = HashMap::new();
            for hct in all_hcts {
                temperatures_by_hct.insert(hct.to_int(), Self::raw_temperature(&hct));
            }
            temperatures_by_hct
        })
    }

    /// Value representing cool-warm factor of a color. Values below 0 are considered cool, above,
    /// warm.
    ///
    /// Implementation of Ou, Woodcock and Wright's algorithm, which uses Lab/LCH color space.
    pub fn raw_temperature(color: &Hct) -> f64 {
        let lab = color.to_int().to_lab();
        let hue = MathUtils::sanitize_degrees_double(lab.b.atan2(lab.a).to_degrees());
        let chroma = lab.a.hypot(lab.b);
        
        -0.5 + 0.02 * chroma.powf(1.07) * (MathUtils::sanitize_degrees_double(hue - 50.0).to_radians()).cos()
    }

    /// Determines if an angle is between two other angles, rotating clockwise.
    fn is_between(angle: f64, a: f64, b: f64) -> bool {
        if a < b {
            a <= angle && angle <= b
        } else {
            a <= angle || angle <= b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hct::hct::Hct;
    use crate::utils::color_utils::Argb;

    #[test]
    fn test_raw_temperature() {
        let blue = Hct::from_int(Argb(0xFF0000FF));
        let red = Hct::from_int(Argb(0xFFFF0000));
        
        assert!(TemperatureCache::raw_temperature(&blue) < TemperatureCache::raw_temperature(&red));
    }

    #[test]
    fn test_complement() {
        // Nice blue
        let blue = Hct::from_int(Argb::from_rgb(12,187,212));
        let cache = TemperatureCache::new(blue);
        let complement = cache.complement();

        // Complement of nice blue should be orangish
        let comp_hue = complement.hue();
        assert!(comp_hue > 50.0);
        assert!(comp_hue < 70.0);
        assert!(TemperatureCache::raw_temperature(&complement) > TemperatureCache::raw_temperature(&blue));
    }

    #[test]
    fn test_analogous_colors() {
        let blue = Hct::from_int(Argb(0xFF0000FF));
        let cache = TemperatureCache::new(blue);
        let analogous = cache.get_analogous_colors();
        
        assert_eq!(analogous.len(), 5);
    }
}
