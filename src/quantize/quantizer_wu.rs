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

use crate::quantize::quantizer::{Quantizer, QuantizerResult};
use crate::quantize::quantizer_map::QuantizerMap;
use crate::utils::color_utils::Argb;
use std::collections::HashMap;

// A histogram of all the input colors is constructed. It has the shape of a cube. The cube
// would be too large if it contained all 16 million colors: historical best practice is to use
// 5 bits of the 8 in each channel, reducing the histogram to a volume of ~32,000.
const INDEX_BITS: u32 = 5;
const INDEX_COUNT: usize = 33; // ((1 << INDEX_BITS) + 1)
const TOTAL_SIZE: usize = 35937; // INDEX_COUNT * INDEX_COUNT * INDEX_COUNT

/// An image quantizer that divides the image's pixels into clusters by recursively cutting an RGB
/// cube, based on the weight of pixels in each area of the cube.
///
/// The algorithm was described by Xiaolin Wu in Graphic Gems II, published in 1991.
pub struct QuantizerWu {
    weights: Vec<i32>,
    moments_r: Vec<i32>,
    moments_g: Vec<i32>,
    moments_b: Vec<i32>,
    moments: Vec<f64>,
    cubes: Vec<Box>,
}

impl Default for QuantizerWu {
    fn default() -> Self {
        Self {
            weights: vec![0; TOTAL_SIZE],
            moments_r: vec![0; TOTAL_SIZE],
            moments_g: vec![0; TOTAL_SIZE],
            moments_b: vec![0; TOTAL_SIZE],
            moments: vec![0.0; TOTAL_SIZE],
            cubes: Vec::new(),
        }
    }
}

impl QuantizerWu {
    pub fn new() -> Self {
        Self::default()
    }

    fn construct_histogram(&mut self, pixels: &HashMap<Argb, u32>) {
        self.weights.fill(0);
        self.moments_r.fill(0);
        self.moments_g.fill(0);
        self.moments_b.fill(0);
        self.moments.fill(0.0);

        for (&pixel, &count) in pixels {
            let red = pixel.red();
            let green = pixel.green();
            let blue = pixel.blue();
            let bits_to_remove = 8 - INDEX_BITS;
            let i_r = (red >> bits_to_remove) + 1;
            let i_g = (green >> bits_to_remove) + 1;
            let i_b = (blue >> bits_to_remove) + 1;
            let index = Self::get_index(i_r as usize, i_g as usize, i_b as usize);

            let count_i = count as i32;
            self.weights[index] += count_i;
            self.moments_r[index] += (red as i32) * count_i;
            self.moments_g[index] += (green as i32) * count_i;
            self.moments_b[index] += (blue as i32) * count_i;
            self.moments[index] += (count as f64)
                * ((red as f64) * (red as f64)
                    + (green as f64) * (green as f64)
                    + (blue as f64) * (blue as f64));
        }
    }

    fn create_moments(&mut self) {
        for r in 1..INDEX_COUNT {
            let mut area = [0i32; INDEX_COUNT];
            let mut area_r = [0i32; INDEX_COUNT];
            let mut area_g = [0i32; INDEX_COUNT];
            let mut area_b = [0i32; INDEX_COUNT];
            let mut area2 = [0.0f64; INDEX_COUNT];
            for g in 1..INDEX_COUNT {
                let mut line = 0i32;
                let mut line_r = 0i32;
                let mut line_g = 0i32;
                let mut line_b = 0i32;
                let mut line2 = 0.0f64;
                for b in 1..INDEX_COUNT {
                    let index = Self::get_index(r, g, b);
                    line += self.weights[index];
                    line_r += self.moments_r[index];
                    line_g += self.moments_g[index];
                    line_b += self.moments_b[index];
                    line2 += self.moments[index];

                    area[b] += line;
                    area_r[b] += line_r;
                    area_g[b] += line_g;
                    area_b[b] += line_b;
                    area2[b] += line2;

                    let previous_index = Self::get_index(r - 1, g, b);
                    self.weights[index] = self.weights[previous_index] + area[b];
                    self.moments_r[index] = self.moments_r[previous_index] + area_r[b];
                    self.moments_g[index] = self.moments_g[previous_index] + area_g[b];
                    self.moments_b[index] = self.moments_b[previous_index] + area_b[b];
                    self.moments[index] = self.moments[previous_index] + area2[b];
                }
            }
        }
    }

    fn create_boxes(&mut self, max_color_count: usize) -> CreateBoxesResult {
        self.cubes = vec![Box::default(); max_color_count];
        let mut volume_variance = vec![0.0; max_color_count];

        let first_box = &mut self.cubes[0];
        first_box.r1 = (INDEX_COUNT - 1) as i32;
        first_box.g1 = (INDEX_COUNT - 1) as i32;
        first_box.b1 = (INDEX_COUNT - 1) as i32;

        let mut generated_color_count = max_color_count;
        let mut next = 0;
        let mut i = 1;
        while i < max_color_count {
            // We need to split the cubes vector to get two mutable references
            let (one, two) = if next < i {
                let (left, right) = self.cubes.split_at_mut(i);
                (&mut left[next], &mut right[0])
            } else {
                let (left, right) = self.cubes.split_at_mut(next);
                (&mut right[0], &mut left[i])
            };

            if Self::cut(
                one,
                two,
                &self.moments_r,
                &self.moments_g,
                &self.moments_b,
                &self.weights,
            ) {
                volume_variance[next] = if one.vol > 1 {
                    Self::variance(
                        one,
                        &self.moments,
                        &self.moments_r,
                        &self.moments_g,
                        &self.moments_b,
                        &self.weights,
                    )
                } else {
                    0.0
                };
                volume_variance[i] = if two.vol > 1 {
                    Self::variance(
                        two,
                        &self.moments,
                        &self.moments_r,
                        &self.moments_g,
                        &self.moments_b,
                        &self.weights,
                    )
                } else {
                    0.0
                };
            } else {
                volume_variance[next] = 0.0;
                i -= 1;
            }

            next = 0;
            let mut temp = volume_variance[0];
            for j in 1..=i {
                if volume_variance[j] > temp {
                    temp = volume_variance[j];
                    next = j;
                }
            }
            if temp <= 0.0 {
                generated_color_count = i + 1;
                break;
            }
            i += 1;
        }

        CreateBoxesResult {
            requested_count: max_color_count as i32,
            result_count: generated_color_count as i32,
        }
    }

    fn create_result(&self, color_count: usize) -> Vec<Argb> {
        let mut colors = Vec::new();
        for i in 0..color_count {
            let cube = &self.cubes[i];
            let weight = Self::volume(cube, &self.weights);
            if weight > 0 {
                let r = Self::volume(cube, &self.moments_r) / weight;
                let g = Self::volume(cube, &self.moments_g) / weight;
                let b = Self::volume(cube, &self.moments_b) / weight;
                let color = Argb::from_rgb((r & 0xFF) as u8, (g & 0xFF) as u8, (b & 0xFF) as u8);
                colors.push(color);
            }
        }
        colors
    }

    fn variance(
        cube: &Box,
        moments: &[f64],
        moments_r: &[i32],
        moments_g: &[i32],
        moments_b: &[i32],
        weights: &[i32],
    ) -> f64 {
        let dr = Self::volume(cube, moments_r);
        let dg = Self::volume(cube, moments_g);
        let db = Self::volume(cube, moments_b);
        let xx = moments[Self::get_index(cube.r1 as usize, cube.g1 as usize, cube.b1 as usize)]
            - moments[Self::get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
            - moments[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
            + moments[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
            - moments[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
            + moments[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
            + moments[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
            - moments[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)];

        let hypotenuse =
            (dr as f64) * (dr as f64) + (dg as f64) * (dg as f64) + (db as f64) * (db as f64);
        let volume = Self::volume(cube, weights);
        xx - (hypotenuse / volume as f64)
    }

    fn cut(
        one: &mut Box,
        two: &mut Box,
        moments_r: &[i32],
        moments_g: &[i32],
        moments_b: &[i32],
        weights: &[i32],
    ) -> bool {
        let whole_r = Self::volume(one, moments_r);
        let whole_g = Self::volume(one, moments_g);
        let whole_b = Self::volume(one, moments_b);
        let whole_w = Self::volume(one, weights);

        let max_r_result = Self::maximize(
            one,
            Direction::Red,
            one.r0 + 1,
            one.r1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
            moments_r,
            moments_g,
            moments_b,
            weights,
        );
        let max_g_result = Self::maximize(
            one,
            Direction::Green,
            one.g0 + 1,
            one.g1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
            moments_r,
            moments_g,
            moments_b,
            weights,
        );
        let max_b_result = Self::maximize(
            one,
            Direction::Blue,
            one.b0 + 1,
            one.b1,
            whole_r,
            whole_g,
            whole_b,
            whole_w,
            moments_r,
            moments_g,
            moments_b,
            weights,
        );

        let max_r = max_r_result.maximum;
        let max_g = max_g_result.maximum;
        let max_b = max_b_result.maximum;

        let cut_direction = if max_r >= max_g && max_r >= max_b {
            if max_r_result.cut_location < 0 {
                return false;
            }
            Direction::Red
        } else if max_g >= max_r && max_g >= max_b {
            Direction::Green
        } else {
            Direction::Blue
        };

        two.r1 = one.r1;
        two.g1 = one.g1;
        two.b1 = one.b1;

        match cut_direction {
            Direction::Red => {
                one.r1 = max_r_result.cut_location;
                two.r0 = one.r1;
                two.g0 = one.g0;
                two.b0 = one.b0;
            }
            Direction::Green => {
                one.g1 = max_g_result.cut_location;
                two.r0 = one.r0;
                two.g0 = one.g1;
                two.b0 = one.b0;
            }
            Direction::Blue => {
                one.b1 = max_b_result.cut_location;
                two.r0 = one.r0;
                two.g0 = one.g0;
                two.b0 = one.b1;
            }
        }

        one.vol = (one.r1 - one.r0) * (one.g1 - one.g0) * (one.b1 - one.b0);
        two.vol = (two.r1 - two.r0) * (two.g1 - two.g0) * (two.b1 - two.b0);

        true
    }

    fn maximize(
        cube: &Box,
        direction: Direction,
        first: i32,
        last: i32,
        whole_r: i32,
        whole_g: i32,
        whole_b: i32,
        whole_w: i32,
        moments_r: &[i32],
        moments_g: &[i32],
        moments_b: &[i32],
        weights: &[i32],
    ) -> MaximizeResult {
        let bottom_r = Self::bottom(cube, direction, moments_r);
        let bottom_g = Self::bottom(cube, direction, moments_g);
        let bottom_b = Self::bottom(cube, direction, moments_b);
        let bottom_w = Self::bottom(cube, direction, weights);

        let mut max = 0.0;
        let mut cut = -1;

        for i in first..last {
            let mut half_r = bottom_r + Self::top(cube, direction, i, moments_r);
            let mut half_g = bottom_g + Self::top(cube, direction, i, moments_g);
            let mut half_b = bottom_b + Self::top(cube, direction, i, moments_b);
            let mut half_w = bottom_w + Self::top(cube, direction, i, weights);

            if half_w == 0 {
                continue;
            }

            let mut temp_numerator = (half_r as f64) * (half_r as f64)
                + (half_g as f64) * (half_g as f64)
                + (half_b as f64) * (half_b as f64);
            let mut temp_denominator = half_w as f64;
            let mut temp = temp_numerator / temp_denominator;

            half_r = whole_r - half_r;
            half_g = whole_g - half_g;
            half_b = whole_b - half_b;
            half_w = whole_w - half_w;

            if half_w == 0 {
                continue;
            }

            temp_numerator = (half_r as f64) * (half_r as f64)
                + (half_g as f64) * (half_g as f64)
                + (half_b as f64) * (half_b as f64);
            temp_denominator = half_w as f64;
            temp += temp_numerator / temp_denominator;

            if temp > max {
                max = temp;
                cut = i;
            }
        }

        MaximizeResult {
            cut_location: cut,
            maximum: max,
        }
    }

    fn get_index(r: usize, g: usize, b: usize) -> usize {
        (r << (INDEX_BITS * 2)) + (r << (INDEX_BITS + 1)) + r + (g << INDEX_BITS) + g + b
    }

    fn volume(cube: &Box, moment: &[i32]) -> i32 {
        moment[Self::get_index(cube.r1 as usize, cube.g1 as usize, cube.b1 as usize)]
            - moment[Self::get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
            - moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
            + moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
            - moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
            + moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
            + moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
            - moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
    }

    fn bottom(cube: &Box, direction: Direction, moment: &[i32]) -> i32 {
        match direction {
            Direction::Red => {
                -moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b1 as usize)]
                    + moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
                    + moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
                    - moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            Direction::Green => {
                -moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b1 as usize)]
                    + moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
                    + moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b1 as usize)]
                    - moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            Direction::Blue => {
                -moment[Self::get_index(cube.r1 as usize, cube.g1 as usize, cube.b0 as usize)]
                    + moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, cube.b0 as usize)]
                    + moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, cube.b0 as usize)]
                    - moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, cube.b0 as usize)]
            }
        }
    }

    fn top(cube: &Box, direction: Direction, position: i32, moment: &[i32]) -> i32 {
        match direction {
            Direction::Red => {
                moment[Self::get_index(position as usize, cube.g1 as usize, cube.b1 as usize)]
                    - moment[Self::get_index(position as usize, cube.g1 as usize, cube.b0 as usize)]
                    - moment[Self::get_index(position as usize, cube.g0 as usize, cube.b1 as usize)]
                    + moment[Self::get_index(position as usize, cube.g0 as usize, cube.b0 as usize)]
            }
            Direction::Green => {
                moment[Self::get_index(cube.r1 as usize, position as usize, cube.b1 as usize)]
                    - moment[Self::get_index(cube.r1 as usize, position as usize, cube.b0 as usize)]
                    - moment[Self::get_index(cube.r0 as usize, position as usize, cube.b1 as usize)]
                    + moment[Self::get_index(cube.r0 as usize, position as usize, cube.b0 as usize)]
            }
            Direction::Blue => {
                moment[Self::get_index(cube.r1 as usize, cube.g1 as usize, position as usize)]
                    - moment[Self::get_index(cube.r1 as usize, cube.g0 as usize, position as usize)]
                    - moment[Self::get_index(cube.r0 as usize, cube.g1 as usize, position as usize)]
                    + moment[Self::get_index(cube.r0 as usize, cube.g0 as usize, position as usize)]
            }
        }
    }
}

impl Quantizer for QuantizerWu {
    fn quantize(&mut self, pixels: &[Argb], max_colors: usize) -> QuantizerResult {
        let mut map_quantizer = QuantizerMap::new();
        let map_result = map_quantizer.quantize(pixels, max_colors);

        self.construct_histogram(&map_result.color_to_count);
        self.create_moments();
        let create_boxes_result = self.create_boxes(max_colors);
        let colors = self.create_result(create_boxes_result.result_count as usize);

        let mut result_map = HashMap::new();
        for color in colors {
            result_map.insert(color, 0);
        }

        QuantizerResult::new(result_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::color_utils::Argb;

    #[test]
    fn test_quantize_wu_basic() {
        let mut quantizer = QuantizerWu::new();
        let pixels = vec![
            Argb::from_rgb(255, 0, 0),
            Argb::from_rgb(0, 255, 0),
            Argb::from_rgb(0, 0, 255),
            Argb::from_rgb(255, 255, 255),
            Argb::from_rgb(0, 0, 0),
        ];
        let result = quantizer.quantize(&pixels, 2);
        assert!(!result.color_to_count.is_empty());
        assert!(result.color_to_count.len() <= 2);
    }

    #[test]
    fn test_quantize_wu_red() {
        let mut quantizer = QuantizerWu::new();
        let pixels = vec![Argb::from_rgb(255, 0, 0); 100];
        let result = quantizer.quantize(&pixels, 10);
        assert_eq!(result.color_to_count.len(), 1);
        let color = result.color_to_count.keys().next().unwrap();
        // Wu quantizer might slightly shift the color due to index mapping
        assert!(color.red() > 240);
        assert!(color.green() < 10);
        assert!(color.blue() < 10);
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Red,
    Green,
    Blue,
}

struct MaximizeResult {
    cut_location: i32,
    maximum: f64,
}

struct CreateBoxesResult {
    #[allow(dead_code)]
    requested_count: i32,
    result_count: i32,
}

#[derive(Default, Clone, Copy)]
struct Box {
    r0: i32,
    r1: i32,
    g0: i32,
    g1: i32,
    b0: i32,
    b1: i32,
    vol: i32,
}
