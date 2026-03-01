use crate::quantize::point_provider::PointProvider;
use crate::quantize::point_provider_lab::PointProviderLab;
use crate::utils::color_utils::Argb;
use indexmap::IndexMap;

#[derive(Clone, Copy)]
struct Distance {
    distance: f64,
}

impl Default for Distance {
    fn default() -> Self {
        Self {
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

    #[must_use]
    pub fn quantize(
        input_pixels: &[Argb],
        starting_clusters: &[Argb],
        max_colors: usize,
    ) -> IndexMap<Argb, u32> {
        let mut random = Random::new(0x42688);
        let point_provider = PointProviderLab;

        // 1. Deduplicate pixels (preserving insertion order via IndexMap)
        let mut pixel_to_count = IndexMap::new();
        for &pixel in input_pixels {
            *pixel_to_count.entry(pixel).or_insert(0) += 1;
        }

        let point_count = pixel_to_count.len();
        if point_count == 0 { return IndexMap::new(); }

        let mut points = Vec::with_capacity(point_count);
        let mut counts = Vec::with_capacity(point_count);

        for (pixel, count) in pixel_to_count {
            points.push(point_provider.point_from_argb(pixel));
            counts.push(count);
        }

        // 2. Initialize clusters
        let mut cluster_count = max_colors.min(point_count);
        if !starting_clusters.is_empty() {
            cluster_count = cluster_count.min(starting_clusters.len());
        }

        let mut clusters = vec![[0.0, 0.0, 0.0]; cluster_count];
        for (i, &starting_argb) in starting_clusters.iter().take(cluster_count).enumerate() {
            clusters[i] = point_provider.point_from_argb(starting_argb);
        }

        let mut cluster_indices: Vec<usize> = (0..point_count)
            .map(|_| random.next_int(cluster_count as i32) as usize)
            .collect();

        let mut distance_to_index_matrix = vec![vec![Distance::default(); cluster_count]; cluster_count];
        let mut pixel_count_sums = vec![0u32; cluster_count];

        // 3. Main Iteration Loop
        for iteration in 0..Self::MAX_ITERATIONS {
            for i in 0..cluster_count {
                for j in i + 1..cluster_count {
                    let distance = point_provider.distance(clusters[i], clusters[j]);
                    distance_to_index_matrix[j][i] = Distance { distance };
                    distance_to_index_matrix[i][j] = Distance { distance };
                }
                distance_to_index_matrix[i].sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal));
            }

            let mut points_moved = 0;
            for i in 0..point_count {
                let point = points[i];
                let previous_cluster_index = cluster_indices[i];
                let previous_distance = point_provider.distance(point, clusters[previous_cluster_index]);

                let mut minimum_distance = previous_distance;
                let mut new_cluster_index = None;

                // CRITICAL: We must maintain this specific loop structure to match original behavior
                for j in 0..cluster_count {
                    if distance_to_index_matrix[previous_cluster_index][j].distance >= 4.0 * previous_distance {
                        continue;
                    }
                    let distance = point_provider.distance(point, clusters[j]);
                    if distance < minimum_distance {
                        minimum_distance = distance;
                        new_cluster_index = Some(j);
                    }
                }

                if let Some(idx) = new_cluster_index {
                    let distance_change = (minimum_distance.sqrt() - previous_distance.sqrt()).abs();
                    if distance_change > Self::MIN_MOVEMENT_DISTANCE {
                        points_moved += 1;
                        cluster_indices[i] = idx;
                    }
                }
            }

            if points_moved == 0 && iteration != 0 {
                break;
            }

            // 4. Update Centroids
            let mut component_a_sums = vec![0.0; cluster_count];
            let mut component_b_sums = vec![0.0; cluster_count];
            let mut component_c_sums = vec![0.0; cluster_count];
            pixel_count_sums.fill(0);

            for i in 0..point_count {
                let cluster_idx = cluster_indices[i];
                let count = counts[i];
                pixel_count_sums[cluster_idx] += count;
                component_a_sums[cluster_idx] += points[i][0] * f64::from(count);
                component_b_sums[cluster_idx] += points[i][1] * f64::from(count);
                component_c_sums[cluster_idx] += points[i][2] * f64::from(count);
            }

            for i in 0..cluster_count {
                let count = pixel_count_sums[i];
                if count > 0 {
                    let c = f64::from(count);
                    clusters[i] = [component_a_sums[i] / c, component_b_sums[i] / c, component_c_sums[i] / c];
                } else {
                    clusters[i] = [0.0, 0.0, 0.0];
                }
            }
        }

        // 5. Final Result Mapping
        clusters.into_iter()
            .zip(pixel_count_sums)
            .filter(|(_, count)| *count > 0)
            .map(|(cluster, count)| (point_provider.point_to_argb(cluster), count))
            .collect()
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
        assert_eq!(result.values().sum::<u32>(), 4);
    }
}
