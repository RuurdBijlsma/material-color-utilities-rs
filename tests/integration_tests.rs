use std::collections::HashMap;
use std::fs;
use std::path::Path;

use material_color_utilities::dynamic::color_spec::SpecVersion;
use material_color_utilities::dynamic::dynamic_scheme::DynamicScheme;
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::utils::color_utils::Argb;

// Import all schemes
use color_eyre::Result;
use material_color_utilities::scheme::scheme_content::SchemeContent;
use material_color_utilities::scheme::scheme_expressive::SchemeExpressive;
use material_color_utilities::scheme::scheme_monochrome::SchemeMonochrome;
use material_color_utilities::scheme::scheme_tonal_spot::SchemeTonalSpot;
use material_color_utilities::scheme::scheme_vibrant::SchemeVibrant;

/// Helper to parse the reference file into a Map: { TestName -> { ColorName -> ArgbValue } }
fn parse_reference_file() -> Result<HashMap<String, HashMap<String, u32>>> {
    let path = Path::new("tests/reference_output.txt");
    println!("File: {}", path.canonicalize()?.exists());
    let content = fs::read_to_string(path)?;

    let mut results = HashMap::new();
    let mut current_test_name = String::new();
    let mut current_colors = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("----------") {
            // New section: save previous if exists
            if !current_test_name.is_empty() {
                results.insert(current_test_name.clone(), current_colors.clone());
            }
            // Extract name from "---------- name ----------"
            current_test_name = line.replace('-', "").trim().to_string();
            current_colors = HashMap::new();
        } else if line.contains(':') {
            // Color line: "primary: 0xFF445E91,"
            let parts: Vec<&str> = line.split(':').collect();
            let key = parts[0].trim().to_string();
            let val_str = parts[1].trim().trim_matches(',');

            // Parse hex string (e.g., 0xFF445E91)
            let val = u32::from_str_radix(val_str.trim_start_matches("0x"), 16)
                .expect(&format!("Failed to parse hex value: {}", val_str));

            current_colors.insert(key, val);
        }
    }

    // Insert last section
    if !current_test_name.is_empty() {
        results.insert(current_test_name, current_colors);
    }

    Ok(results)
}

/// Maps a test name to the corresponding Rust Scheme object.
/// Source colors must match your Kotlin main() values.
fn get_scheme_by_name(name: &str) -> DynamicScheme {
    match name {
        "test_tonal_spot_light" => {
            SchemeTonalSpot::new(Hct::from_int(Argb(0xFF4285F4)), false, 0.0)
        }
        "test_tonal_spot_dark" => SchemeTonalSpot::new(Hct::from_int(Argb(0xFF4285F4)), true, 0.0),

        "test_content_light" => SchemeContent::new(Hct::from_int(Argb(0xFFEA4335)), false, 0.0),
        "test_content_dark" => SchemeContent::new(Hct::from_int(Argb(0xFFEA4335)), true, 0.0),

        "test_vibrant_light" => SchemeVibrant::new(Hct::from_int(Argb(0xFFFBBC04)), false, 0.0),
        "test_vibrant_dark" => SchemeVibrant::new(Hct::from_int(Argb(0xFFFBBC04)), true, 0.0),

        "test_monochrome_light" => {
            SchemeMonochrome::new(Hct::from_int(Argb(0xFF34A853)), false, 0.0)
        }
        "test_monochrome_dark" => SchemeMonochrome::new(Hct::from_int(Argb(0xFF34A853)), true, 0.0),

        "test_expressive_light" => {
            SchemeExpressive::new(Hct::from_int(Argb(0xFF6200EE)), false, 0.0)
        }
        "test_expressive_dark" => SchemeExpressive::new(Hct::from_int(Argb(0xFF6200EE)), true, 0.0),

        _ => panic!("Unknown test scheme name in reference file: {}", name),
    }
}

/// Helper to get a color from MaterialDynamicColors by string name.
fn get_color_argb(mdc: &MaterialDynamicColors, scheme: &DynamicScheme, color_name: &str) -> u32 {
    let dynamic_color = match color_name {
        "primary" => mdc.primary(),
        "on_primary" => mdc.on_primary(),
        "primary_container" => mdc.primary_container(),
        "on_primary_container" => mdc.on_primary_container(),
        "inverse_primary" => mdc.inverse_primary(),
        "secondary" => mdc.secondary(),
        "on_secondary" => mdc.on_secondary(),
        "secondary_container" => mdc.secondary_container(),
        "on_secondary_container" => mdc.on_secondary_container(),
        "tertiary" => mdc.tertiary(),
        "on_tertiary" => mdc.on_tertiary(),
        "tertiary_container" => mdc.tertiary_container(),
        "on_tertiary_container" => mdc.on_tertiary_container(),
        "error" => mdc.error(),
        "on_error" => mdc.on_error(),
        "error_container" => mdc.error_container(),
        "on_error_container" => mdc.on_error_container(),
        "background" => mdc.background(),
        "on_background" => mdc.on_background(),
        "surface" => mdc.surface(),
        "on_surface" => mdc.on_surface(),
        "surface_variant" => mdc.surface_variant(),
        "on_surface_variant" => mdc.on_surface_variant(),
        "inverse_surface" => mdc.inverse_surface(),
        "inverse_on_surface" => mdc.inverse_on_surface(),
        "surface_dim" => mdc.surface_dim(),
        "surface_bright" => mdc.surface_bright(),
        "surface_container_lowest" => mdc.surface_container_lowest(),
        "surface_container_low" => mdc.surface_container_low(),
        "surface_container" => mdc.surface_container(),
        "surface_container_high" => mdc.surface_container_high(),
        "surface_container_highest" => mdc.surface_container_highest(),
        "outline" => mdc.outline(),
        "outline_variant" => mdc.outline_variant(),
        "shadow" => mdc.shadow(),
        "scrim" => mdc.scrim(),
        "surface_tint" => mdc.surface_tint(),
        _ => panic!("Unknown color method name: {}", color_name),
    };

    dynamic_color.get_argb(scheme).0
}

#[test]
fn test_material_schemes_against_reference() -> Result<()> {
    let references = parse_reference_file()?;

    // Kotlin's MaterialDynamicColors() defaults to Spec2021 logic in older versions
    // or Spec2026 in newest. Based on your Kotlin snippet comments, we'll try 2021
    // first to match your previous test logic, but you can change this to 2026.
    let mdc = MaterialDynamicColors::new_with_spec(SpecVersion::Spec2021);

    for (test_name, expected_colors) in references {
        println!("Running reference check for: {}", test_name);
        let scheme = get_scheme_by_name(&test_name);

        for (color_name, &expected_argb) in &expected_colors {
            let actual_argb = get_color_argb(&mdc, &scheme, color_name);

            println!(
                "\nTest: {}\nColor: {}\nExpected: {:#010X}\nActual:   {:#010X}",
                test_name, color_name, actual_argb, expected_argb
            );
            assert_eq!(
                actual_argb, expected_argb,
                "\nTest: {}\nColor: {}\nExpected: {:#010X}\nActual:   {:#010X}",
                test_name, color_name, expected_argb, actual_argb
            );
        }
    }

    Ok(())
}
