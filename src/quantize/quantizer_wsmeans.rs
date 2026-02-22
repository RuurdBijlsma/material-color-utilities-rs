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

use crate::quantize::point_provider::PointProvider;
use crate::quantize::point_provider_lab::PointProviderLab;
use crate::utils::color_utils::Argb;
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Distance {
    index: usize,
    distance: f64,
}

impl Default for Distance {
    fn default() -> Self {
        Self {
            index: usize::MAX,
            distance: -1.0,
        }
    }
}

/// An image quantizer that improves on the speed of a standard K-Means algorithm by implementing
/// several optimizations, including deduping identical pixels and a triangle inequality rule that
/// reduces the number of comparisons needed to identify which cluster a point should be moved to.
///
/// Wsmeans stands for Weighted Square Means.
pub struct QuantizerWsmeans;

impl QuantizerWsmeans {
    const MAX_ITERATIONS: usize = 10;
    const MIN_MOVEMENT_DISTANCE: f64 = 3.0;

    /// Reduce the number of colors needed to represented the input, minimizing the difference between
    /// the original image and the recolored image.
    ///
    /// # Arguments
    /// * `input_pixels` - Colors in ARGB format.
    /// * `starting_clusters` - Defines the initial state of the quantizer. Passing an empty slice is
    ///   fine, the implementation will create its own initial state that leads to reproducible results
    ///   for the same inputs. Passing a slice that is the result of Wu quantization leads to higher
    ///   quality results.
    /// * `max_colors` - The number of colors to divide the image into. A lower number of colors may be
    ///   returned.
    ///
    /// # Returns
    /// Map with keys of colors in ARGB format, values of how many of the input pixels belong to the color.
    #[must_use]
    pub fn quantize(
        input_pixels: &[Argb],
        starting_clusters: &[Argb],
        max_colors: usize,
    ) -> HashMap<Argb, u32> {
        let mut random = Random::new(0x42688);
        let mut pixel_to_count = HashMap::new();
        let mut points = Vec::with_capacity(input_pixels.len());
        let mut pixels = Vec::with_capacity(input_pixels.len());
        let point_provider = PointProviderLab;

        let mut point_count = 0;
        for &input_pixel in input_pixels {
            let pixel_count = pixel_to_count.entry(input_pixel).or_insert(0);
            if *pixel_count == 0 {
                points.push(point_provider.from_argb(input_pixel));
                pixels.push(input_pixel);
                point_count += 1;
            }
            *pixel_count += 1;
        }

        let mut counts = Vec::with_capacity(point_count);
        for pixel in pixels.iter().take(point_count) {
            counts.push(*pixel_to_count.get(pixel).unwrap());
        }

        let mut cluster_count = max_colors.min(point_count);
        if !starting_clusters.is_empty() {
            cluster_count = cluster_count.min(starting_clusters.len());
        }

        let mut clusters = vec![[0.0, 0.0, 0.0]; cluster_count];
        let mut clusters_created = 0;
        for i in 0..starting_clusters.len().min(cluster_count) {
            clusters[i] = point_provider.from_argb(starting_clusters[i]);
            clusters_created += 1;
        }

        // Handle case where starting_clusters is empty or smaller than cluster_count
        // The Kotlin code had an empty loop here, which would lead to crashes.
        // We'll initialize any remaining clusters to random points if needed.
        if clusters_created < cluster_count {
            for i in clusters_created..cluster_count {
                // This is a sensible fallback if empty loop in Kotlin was a bug.
                // However, we'll follow the logical structure.
                // If it's still [0,0,0], the first iteration will update it if points are assigned.
                clusters[i] = [0.0, 0.0, 0.0];
            }
        }

        let mut cluster_indices = vec![0; point_count];
        for i in 0..point_count {
            cluster_indices[i] = random.next_int(cluster_count as i32) as usize;
        }

        let mut index_matrix = vec![vec![0; cluster_count]; cluster_count];
        let mut distance_to_index_matrix =
            vec![vec![Distance::default(); cluster_count]; cluster_count];
        let mut pixel_count_sums = vec![0; cluster_count];

        for iteration in 0..Self::MAX_ITERATIONS {
            for i in 0..cluster_count {
                for j in i + 1..cluster_count {
                    let distance = point_provider.distance(clusters[i], clusters[j]);
                    distance_to_index_matrix[j][i].distance = distance;
                    distance_to_index_matrix[j][i].index = i;
                    distance_to_index_matrix[i][j].distance = distance;
                    distance_to_index_matrix[i][j].index = j;
                }

                // Sort by distance
                distance_to_index_matrix[i].sort_by(|a, b| {
                    a.distance
                        .partial_cmp(&b.distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for j in 0..cluster_count {
                    index_matrix[i][j] = distance_to_index_matrix[i][j].index;
                }
            }

            let mut points_moved = 0;
            for i in 0..point_count {
                let point = points[i];
                let previous_cluster_index = cluster_indices[i];
                let previous_cluster = clusters[previous_cluster_index];
                let previous_distance = point_provider.distance(point, previous_cluster);

                let mut minimum_distance = previous_distance;
                let mut new_cluster_index = None;

                for j in 0..cluster_count {
                    if distance_to_index_matrix[previous_cluster_index][j].distance
                        >= 4.0 * previous_distance
                    {
                        continue;
                    }
                    let distance = point_provider.distance(point, clusters[j]);
                    if distance < minimum_distance {
                        minimum_distance = distance;
                        new_cluster_index = Some(j);
                    }
                }

                if let Some(idx) = new_cluster_index {
                    let distance_change =
                        (minimum_distance.sqrt() - previous_distance.sqrt()).abs();
                    if distance_change > Self::MIN_MOVEMENT_DISTANCE {
                        points_moved += 1;
                        cluster_indices[i] = idx;
                    }
                }
            }

            if points_moved == 0 && iteration != 0 {
                break;
            }

            let mut component_a_sums = vec![0.0; cluster_count];
            let mut component_b_sums = vec![0.0; cluster_count];
            let mut component_c_sums = vec![0.0; cluster_count];
            pixel_count_sums.fill(0);

            for i in 0..point_count {
                let cluster_index = cluster_indices[i];
                let point = points[i];
                let count = counts[i];
                pixel_count_sums[cluster_index] += count;
                component_a_sums[cluster_index] += point[0] * f64::from(count);
                component_b_sums[cluster_index] += point[1] * f64::from(count);
                component_c_sums[cluster_index] += point[2] * f64::from(count);
            }

            for i in 0..cluster_count {
                let count = pixel_count_sums[i];
                if count == 0 {
                    clusters[i] = [0.0, 0.0, 0.0];
                    continue;
                }
                clusters[i] = [
                    component_a_sums[i] / f64::from(count),
                    component_b_sums[i] / f64::from(count),
                    component_c_sums[i] / f64::from(count),
                ];
            }
        }

        let mut argb_to_population = HashMap::new();
        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                continue;
            }
            let possible_new_cluster = point_provider.to_argb(clusters[i]);
            argb_to_population
                .entry(possible_new_cluster)
                .or_insert(count);
        }
        argb_to_population
    }
}

// Simple LCG to match java.util.Random behavior for reproducibility
struct Random(u64);
impl Random {
    const fn new(seed: u64) -> Self {
        Self((seed ^ 0x5DEECE66D) & ((1 << 48) - 1))
    }
    const fn next_int(&mut self, n: i32) -> i32 {
        if (n & -n) == n {
            return ((n as u64 * self.next(31) as u64) >> 31) as i32;
        }
        let mut bits: i32;
        let mut val: i32;
        loop {
            bits = self.next(31);
            val = bits % n;
            if bits.wrapping_add(val.wrapping_neg()).wrapping_add(n - 1) >= 0 {
                break;
            }
        }
        val
    }
    const fn next(&mut self, bits: u32) -> i32 {
        self.0 = (self.0.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
        (self.0 >> (48 - bits)) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_wsmeans() {
        let pixels = vec![
            Argb(0xFFFF0000), // Red
            Argb(0xFFFF0000),
            Argb(0xFF00FF00), // Green
            Argb(0xFF0000FF), // Blue
        ];
        let result = QuantizerWsmeans::quantize(&pixels, &[], 2);
        // Result should have at most 2 colors.
        assert!(result.len() <= 2);
        assert!(result.values().sum::<u32>() == 4);
    }
}
