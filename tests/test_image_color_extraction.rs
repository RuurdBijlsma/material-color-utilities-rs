use color_eyre::Result;
use color_eyre::eyre::{eyre, Context};
use material_color_utilities::hct::Cam16;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::quantize::{Quantizer, QuantizerCelebi};
use material_color_utilities::score::score_colors::Score;
use material_color_utilities::utils::color_utils::Argb;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
struct ReferenceCase {
    image: String,
    settings: Settings,
    seeds: Vec<String>,
}

#[derive(Deserialize)]
struct Settings {
    max_colors: usize,
    desired_count: usize,
}

struct ExtractionMismatch {
    image_name: String,
    max_colors: usize,
    index: usize,
    expected: Argb,
    actual: Argb,
    distance: f64,
}

#[derive(Default)]
struct ExtractionTracker {
    mismatches: Vec<ExtractionMismatch>,
    total_images_processed: usize,
    total_seeds_checked: usize,
    all_distances_by_image: HashMap<String, Vec<f64>>,
}

fn calculate_stats(distances: &[f64]) -> (f64, f64) {
    if distances.is_empty() {
        return (0.0, 0.0);
    }
    let count = distances.len() as f64;
    let mean = distances.iter().sum::<f64>() / count;
    let variance = distances.iter()
        .map(|d| (d - mean).powi(2))
        .sum::<f64>() / count;
    (mean, variance.sqrt())
}

impl ExtractionTracker {
    fn report(&self) {
        let (global_mean, global_std_dev) = calculate_stats(
            &self.all_distances_by_image.values().flatten().cloned().collect::<Vec<_>>()
        );

        println!("\n================================ COLOR EXTRACTION REPORT ================================");
        println!("Global Context: {} mismatches found out of {} total checks across {} images.",
                 self.mismatches.len(), self.total_seeds_checked, self.total_images_processed);
        println!("Global Accuracy: {:.2}%",
                 (1.0 - (self.mismatches.len() as f64 / self.total_seeds_checked as f64)) * 100.0);
        println!("Global Error Stats: Mean ΔE: {:.4}, StdDev: {:.4}", global_mean, global_std_dev);

        if self.mismatches.is_empty() {
            return;
        }

        // Group mismatches by image
        let mut by_image: HashMap<String, Vec<&ExtractionMismatch>> = HashMap::new();
        for m in &self.mismatches {
            by_image.entry(m.image_name.clone()).or_default().push(m);
        }

        let mut sorted_keys: Vec<_> = by_image.keys().collect();
        sorted_keys.sort();

        for img_name in sorted_keys {
            let ms = &by_image[img_name];
            let img_distances = &self.all_distances_by_image[img_name];
            let (img_mean, img_std_dev) = calculate_stats(img_distances);

            println!("\n🖼️  Image: {:<15} | Avg ΔE: {:>6.2} | StdDev: {:>6.2}", img_name, img_mean, img_std_dev);
            println!("{:-<115}", "");
            println!("{:<5} | {:<8} | {:<10} | {:<10} | {:<22} | {:<22} | {:<6}",
                     "Rank", "MaxCol", "Expected", "Actual", "Expected HCT", "Actual HCT", "ΔE");
            println!("{:-<115}", "");

            for m in ms {
                let exp_hct = Hct::from_int(m.expected);
                let act_hct = Hct::from_int(m.actual);

                let exp_hct_str = format!("{:.1}, {:.1}, {:.1}", exp_hct.hue(), exp_hct.chroma(), exp_hct.tone());
                let act_hct_str = format!("{:.1}, {:.1}, {:.1}", act_hct.hue(), act_hct.chroma(), act_hct.tone());

                println!(
                    "#{:<4} | {:<8} | 0x{:08X} | 0x{:08X} | {:<22} | {:<22} | {:>6.2}",
                    m.index,
                    m.max_colors,
                    m.expected.0,
                    m.actual.0,
                    exp_hct_str,
                    act_hct_str,
                    m.distance
                );
            }
        }
        println!("\n=========================================================================================");
    }
}

#[test]
fn test_color_extraction() -> Result<()> {
    let json_data = fs::read_to_string("tests/assets/json/reference_extraction.json")
        .wrap_err("Unable to read reference file")?;
    let cases: Vec<ReferenceCase> = serde_json::from_str(&json_data)?;

    let mut tracker = ExtractionTracker::default();

    for case in cases {
        tracker.total_images_processed += 1;

        let img_path = format!("tests/assets/img/{}", case.image);
        let img = image::open(&img_path).wrap_err_with(|| format!("Failed to open {}", img_path))?;

        let pixels: Vec<Argb> = img
            .to_rgba8()
            .pixels()
            .map(|p| {
                let [r, g, b, _] = p.0;
                Argb::from_rgb(r, g, b)
            })
            .collect();

        // 1. Quantize
        let mut celebi = QuantizerCelebi::new();
        let result = celebi.quantize(&pixels, case.settings.max_colors);

        // 2. Score
        let seeds = Score::score_desired(&result.color_to_count, case.settings.desired_count);

        // 3. Parse Expected
        let expected_seeds: Vec<Argb> = case.seeds.iter()
            .map(|s| {
                let val = u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap();
                Argb(val)
            })
            .collect();

        // 4. Compare
        for (i, &expected) in expected_seeds.iter().enumerate() {
            tracker.total_seeds_checked += 1;

            let actual_argb = seeds.get(i).copied().unwrap_or(Argb(0));

            // Perceptual distance check
            let dist = if actual_argb == Argb(0) {
                999.0 // Sentinel for missing color
            } else {
                Cam16::from_int(expected).distance(&Cam16::from_int(actual_argb))
            };

            // Track distance for image stats regardless of whether it's a mismatch
            tracker.all_distances_by_image.entry(case.image.clone())
                .or_default()
                .push(dist);

            if actual_argb != expected {
                tracker.mismatches.push(ExtractionMismatch {
                    image_name: case.image.clone(),
                    max_colors: case.settings.max_colors,
                    index: i,
                    expected,
                    actual: actual_argb,
                    distance: dist,
                });
            }
        }
    }

    tracker.report();

    if !tracker.mismatches.is_empty() {
        return Err(eyre!("Color extraction mismatches found. See report above."));
    }

    Ok(())
}