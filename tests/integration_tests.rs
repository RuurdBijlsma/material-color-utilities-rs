use std::collections::HashMap;
use std::fs;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::Deserialize;

use material_color_utilities::dynamic::color_spec::SpecVersion;
use material_color_utilities::dynamic::dynamic_scheme::DynamicScheme;
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::utils::color_utils::Argb;

use material_color_utilities::scheme::scheme_content::SchemeContent;
use material_color_utilities::scheme::scheme_expressive::SchemeExpressive;
use material_color_utilities::scheme::scheme_monochrome::SchemeMonochrome;
use material_color_utilities::scheme::scheme_tonal_spot::SchemeTonalSpot;
use material_color_utilities::scheme::scheme_vibrant::SchemeVibrant;
use material_color_utilities::scheme::{
    SchemeCmf, SchemeFidelity, SchemeFruitSalad, SchemeNeutral, SchemeRainbow,
};

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
                                "Mismatch for scheme {} role {}: expected {:#010X}, got {:#010X}",
                                entry.scheme, role_name, expected, actual
                            ));
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
        println!("\n\n\n✅ No missing_scheme, {tested_scheme} checks done");
    } else {
        eprintln!(
            "\n\n\nFound {}/{} missing_scheme:",
            missing_scheme.len(),
            tested_scheme
        );
        // for m in &missing_scheme {
        //     eprintln!("{m}");
        // }
    }

    if invalid_hex.is_empty() {
        println!("\n\n\n✅ No invalid_hex, {tested_hex} checks done");
    } else {
        eprintln!(
            "\n\n\nFound {}/{} invalid_hex:",
            invalid_hex.len(),
            tested_hex
        );
        // for m in &invalid_hex {
        //     eprintln!("{m}");
        // }
    }

    if mismatch_color.is_empty() {
        println!("\n\n\n✅ No mismatch_color, {tested_color} checks done");
    } else {
        eprintln!(
            "\n\n\nFound {}/{} mismatch_color:",
            mismatch_color.len(),
            tested_color
        );
        // for m in &mismatch_color {
        //     eprintln!("{m}");
        // }
    }

    if missing_role.is_empty() {
        println!("\n\n\n✅ No missing_role, {tested_role} checks done");
    } else {
        eprintln!(
            "\n\n\nFound {}/{} missing_role:",
            missing_role.len(),
            tested_role
        );
        // for m in &missing_role {
        //     eprintln!("{m}");
        // }
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
