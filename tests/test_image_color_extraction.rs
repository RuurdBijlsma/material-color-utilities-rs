use color_eyre::Result;
use image::{GenericImageView, imageops::FilterType};
use material_color_utilities::quantize::{Quantizer, QuantizerCelebi};
use material_color_utilities::score::score_colors::Score;
use material_color_utilities::utils::color_utils::Argb;
use serde::Deserialize;
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

#[test]
fn test_color_extraction() -> Result<()> {
    let json_data = fs::read_to_string("tests/assets/json/reference_extraction.json")
        .expect("Unable to read reference file");
    let cases: Vec<ReferenceCase> = serde_json::from_str(&json_data)?;

    for case in cases {
        // 1. Load the same image
        let img_path = format!("tests/assets/img/{}", case.image);
        let img = image::open(&img_path)?;

        let pixels: Vec<Argb> = img
            .to_rgba8() // Ensure we have 4 channels
            .pixels()
            .map(|p| {
                let [r, g, b, _] = p.0;
                Argb::from_rgb(r, g, b)
            })
            .collect();

        // 4. Quantize
        let mut celebi = QuantizerCelebi::new();
        let result = celebi.quantize(&pixels, case.settings.max_colors);

        // 5. Score
        let seeds = Score::score_desired(&result.color_to_count, case.settings.desired_count);

        // 6. Compare
        let rust_hex_seeds: Vec<String> = seeds.iter().map(|s| format!("0x{:08X}", s.0)).collect();

        assert_eq!(
            rust_hex_seeds, case.seeds,
            "Mismatch in image {} with max_colors {}",
            case.image, case.settings.max_colors
        );
    }

    Ok(())
}
