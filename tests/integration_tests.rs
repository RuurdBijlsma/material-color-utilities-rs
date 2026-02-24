use std::collections::HashMap;
use std::fs;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use plotters::prelude::{BLUE, BitMapBackend, ChartBuilder, IntoDrawingArea, LineSeries, WHITE};
use serde::Deserialize;

use material_color_utilities::dynamic::color_spec::SpecVersion;
use material_color_utilities::dynamic::dynamic_scheme::DynamicScheme;
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;
use material_color_utilities::hct::Cam16;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::scheme::scheme_content::SchemeContent;
use material_color_utilities::scheme::scheme_expressive::SchemeExpressive;
use material_color_utilities::scheme::scheme_monochrome::SchemeMonochrome;
use material_color_utilities::scheme::scheme_tonal_spot::SchemeTonalSpot;
use material_color_utilities::scheme::scheme_vibrant::SchemeVibrant;
use material_color_utilities::scheme::{
    SchemeCmf, SchemeFidelity, SchemeFruitSalad, SchemeNeutral, SchemeRainbow,
};
use material_color_utilities::utils::color_utils::Argb;
use material_color_utilities::utils::string_utils::StringUtils;
use plotters::prelude::*;
use rand::prelude::IndexedRandom;
use statrs::statistics::Statistics;

#[derive(Debug, Deserialize)]
struct ReferenceEntry {
    color: String,
    scheme: String,
    contrast: f64,
    is_dark: bool,
    roles: HashMap<String, String>,
}

fn parse_reference_schemes() -> Result<Vec<ReferenceEntry>> {
    let content = fs::read_to_string("tests/reference_schemes.json")?;
    let entries: Vec<ReferenceEntry> = serde_json::from_str(&content)?;
    Ok(entries)
}

fn make_scheme_from_entry(entry: &ReferenceEntry) -> Option<DynamicScheme> {
    // Parse color hex like "0xFFDCDCDC" -> 0xFFDCDCDC
    let Ok(color_val) = u32::from_str_radix(entry.color.trim_start_matches("0x"), 16) else {
        return None;
    };

    match entry.scheme.as_str() {
        "CMF" => Some(SchemeCmf::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "CONTENT" => Some(SchemeContent::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "EXPRESSIVE" => Some(SchemeExpressive::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "FIDELITY" => Some(SchemeFidelity::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "FRUIT_SALAD" => Some(SchemeFruitSalad::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "MONOCHROME" => Some(SchemeMonochrome::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "NEUTRAL" => Some(SchemeNeutral::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "RAINBOW" => Some(SchemeRainbow::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "TONAL_SPOT" => Some(SchemeTonalSpot::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        "VIBRANT" => Some(SchemeVibrant::new(
            Hct::from_int(Argb(color_val)),
            entry.is_dark,
            entry.contrast,
        )),
        _ => None,
    }
}

pub fn plot_debug(data: &[f64], title: &str, y_label: &str) -> Result<()> {
    let root = BitMapBackend::new("plot.png", (10000, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_range = 0..data.len();
    let (y_min, y_max) = data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });

    let padding = (y_max - y_min).abs() * 0.1;
    let y_range = (y_min - padding)..(y_max + padding);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 28))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_range.clone(), y_range)?;

    chart
        .configure_mesh()
        .x_desc("Index")
        .y_desc(y_label)
        .light_line_style(&RGBColor(220, 220, 220))
        .bold_line_style(&RGBColor(180, 180, 180))
        .label_style(("sans-serif", 14))
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().enumerate().map(|(i, v)| (i, *v)),
        ShapeStyle::from(&BLUE).stroke_width(2),
    ))?;

    // // Optional: highlight points
    // chart.draw_series(
    //     data.iter()
    //         .enumerate()
    //         .map(|(i, v)| Circle::new((i, *v), 3, BLUE.filled())),
    // )?;

    root.present()?;
    Ok(())
}

#[test]
fn test_material_schemes_against_reference() -> Result<()> {
    let entries = parse_reference_schemes()?;

    let mdc = MaterialDynamicColors::new_with_spec(SpecVersion::Spec2026);
    let mut invalid_hex: Vec<String> = Vec::new();
    let mut mismatch_color: Vec<String> = Vec::new();
    let mut missing_role: Vec<String> = Vec::new();
    let mut missing_scheme: Vec<String> = Vec::new();
    let mut tested_hex = 0;
    let mut tested_color = 0;
    let mut tested_role = 0;
    let mut tested_scheme = 0;

    // Explicitly type to take ownership of String keys rather than references
    let mut role_mismatch_count_map: HashMap<String, usize> = HashMap::new();
    let mut scheme_mismatch_count_map: HashMap<String, usize> = HashMap::new();
    let mut color_mismatch_distances = Vec::new();

    for entry in entries {
        tested_scheme += 1;
        if let Some(scheme) = make_scheme_from_entry(&entry) {
            // Build actual roles map from the Rust MaterialDynamicColors
            let mut actual_roles: HashMap<String, u32> = HashMap::new();
            for getter in mdc.all_dynamic_colors() {
                if let Some(dc) = getter() {
                    actual_roles.insert(dc.name.clone(), dc.get_argb(&scheme).0);
                }
            }

            for (role_name, hex_str) in entry.roles {
                tested_hex += 1;
                let Ok(expected) = u32::from_str_radix(hex_str.trim_start_matches("0x"), 16) else {
                    invalid_hex.push(format!(
                        "Invalid hex for role {role_name} in reference: {hex_str}"
                    ));
                    continue;
                };

                tested_role += 1;
                match actual_roles.get(&role_name).copied() {
                    Some(actual) => {
                        tested_color += 1;
                        if actual != expected {
                            mismatch_color.push(format!(
                                "scheme: {}, role: {}, expected: {}, got: {}",
                                &entry.scheme,
                                role_name,
                                Hct::from_int(Argb(expected)),
                                Hct::from_int(Argb(actual))
                            ));

                            // The entry API is the idiomatic way to handle map counts in Rust
                            // It passes a cloned owned String, resolving all borrow checker errors.
                            *role_mismatch_count_map
                                .entry(role_name.clone())
                                .or_insert(0) += 1;
                            *scheme_mismatch_count_map
                                .entry(entry.scheme.clone())
                                .or_insert(0) += 1;

                            let col1 = Cam16::from_int(Argb(expected));
                            let col2 = Cam16::from_int(Argb(actual));
                            let distance = col1.distance(&col2);
                            color_mismatch_distances.push(distance);
                        }
                    }
                    None => {
                        missing_role.push(format!(
                            "Role {} not found in MaterialDynamicColors for scheme {}",
                            role_name, entry.scheme
                        ));
                    }
                }
            }
        } else {
            missing_scheme.push(format!("Scheme {} is unsupported", entry.scheme));
        }
    }

    if missing_scheme.is_empty() {
        println!("\n✅ No missing_scheme, {tested_scheme} checks done");
    } else {
        eprintln!(
            "\nFound {}/{} missing_scheme:",
            missing_scheme.len(),
            tested_scheme
        );
    }

    if invalid_hex.is_empty() {
        println!("\n✅ No invalid_hex, {tested_hex} checks done");
    } else {
        eprintln!("\nFound {}/{} invalid_hex:", invalid_hex.len(), tested_hex);
    }

    if mismatch_color.is_empty() {
        println!("\n✅ No mismatch_color, {tested_color} checks done");
    } else {
        eprintln!(
            "\n⛔ Found {}/{} mismatch_color:",
            mismatch_color.len(),
            tested_color
        );
        let mut rng = rand::rng();
        let sample: Vec<&String> = mismatch_color.sample(&mut rng, 15).collect();
        eprintln!("Sample of {} random mismatches", sample.len());
        for m in &sample {
            eprintln!("\t{m}");
        }
        eprintln!(
            "\nMismatches per scheme:\n{}",
            serde_json::to_string_pretty(&scheme_mismatch_count_map)?
        );
        eprintln!(
            "Mismatches per role:\n{}",
            serde_json::to_string_pretty(&role_mismatch_count_map)?
        );
        eprintln!("Mismatch color distance stats:");
        eprintln!("\t* Mean: {}", (&color_mismatch_distances).mean());
        eprintln!("\t* Stddev: {}", (&color_mismatch_distances).std_dev());
        let black = Cam16::from_int(Argb::from_rgb(0, 0, 0));
        let white = Cam16::from_int(Argb::from_rgb(255, 255, 255));
        let yellow = Cam16::from_int(Argb::from_rgb(255, 251, 0));
        let orange = Cam16::from_int(Argb::from_rgb(255, 164, 0));
        let red = Cam16::from_int(Argb::from_rgb(255, 0, 0));
        eprintln!("black <-> white Distance: {}", black.distance(&white));
        eprintln!("yellow <-> white Distance: {}", yellow.distance(&white));
        eprintln!("yellow <-> orange Distance: {}", yellow.distance(&orange));
        eprintln!("red <-> orange Distance: {}", red.distance(&orange));
        plot_debug(
            &color_mismatch_distances,
            "Mismatch Color Distances",
            "Distance in CAM16 space",
        )?;
    }

    if missing_role.is_empty() {
        println!("\n✅ No missing_role, {tested_role} checks done");
    } else {
        eprintln!(
            "\nFound {}/{} missing_role:",
            missing_role.len(),
            tested_role
        );
    }

    if !missing_scheme.is_empty()
        || !missing_role.is_empty()
        || !invalid_hex.is_empty()
        || !mismatch_color.is_empty()
    {
        return Err(eyre!("Test failed"));
    }

    Ok(())
}
