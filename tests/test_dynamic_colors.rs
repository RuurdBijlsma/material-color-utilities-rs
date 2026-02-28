use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use material_color_utilities::dynamic::color_spec::SpecVersion;
use material_color_utilities::dynamic::dynamic_scheme::DynamicScheme;
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;
use material_color_utilities::hct::Cam16;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::scheme::{
    SchemeCmf, SchemeFidelity, SchemeFruitSalad, SchemeNeutral, SchemeRainbow,
    scheme_content::SchemeContent, scheme_expressive::SchemeExpressive,
    scheme_monochrome::SchemeMonochrome, scheme_tonal_spot::SchemeTonalSpot,
    scheme_vibrant::SchemeVibrant,
};
use material_color_utilities::utils::color_utils::Argb;
use rand::prelude::IndexedRandom;
use serde::Deserialize;
use statrs::statistics::Statistics;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct ReferenceEntry {
    color: String,
    scheme: String,
    contrast: f64,
    #[serde(rename = "is_dark")]
    is_dark: bool,
    roles: HashMap<String, String>,
}

impl ReferenceEntry {
    fn parse_color(&self, hex: &str) -> Result<Argb> {
        let val = u32::from_str_radix(hex.trim_start_matches("0x"), 16)
            .map_err(|_| eyre!("Invalid hex format: {}", hex))?;
        Ok(Argb(val))
    }

    fn to_dynamic_scheme(&self) -> Result<DynamicScheme> {
        let hct = Hct::from_int(self.parse_color(&self.color)?);
        let d = self.is_dark;
        let c = self.contrast;

        match self.scheme.as_str() {
            "CMF" => Ok(SchemeCmf::new(hct, d, c)),
            "CONTENT" => Ok(SchemeContent::new(hct, d, c)),
            "EXPRESSIVE" => Ok(SchemeExpressive::new(hct, d, c)),
            "FIDELITY" => Ok(SchemeFidelity::new(hct, d, c)),
            "FRUIT_SALAD" => Ok(SchemeFruitSalad::new(hct, d, c)),
            "MONOCHROME" => Ok(SchemeMonochrome::new(hct, d, c)),
            "NEUTRAL" => Ok(SchemeNeutral::new(hct, d, c)),
            "RAINBOW" => Ok(SchemeRainbow::new(hct, d, c)),
            "TONAL_SPOT" => Ok(SchemeTonalSpot::new(hct, d, c)),
            "VIBRANT" => Ok(SchemeVibrant::new(hct, d, c)),
            _ => Err(eyre!("Unsupported scheme type: {}", self.scheme)),
        }
    }
}

#[derive(Default)]
struct ValidationTracker {
    mismatches: Vec<String>,
    distances: Vec<f64>,
    role_counts: HashMap<String, usize>,
    scheme_counts: HashMap<String, usize>,
    total_tested: usize,
}

impl ValidationTracker {
    fn record_mismatch(&mut self, role: &str, scheme: &str, expected: Argb, actual: Argb) {
        let d = Cam16::from_int(expected).distance(&Cam16::from_int(actual));
        self.distances.push(d);

        let fmt_hct = |argb: Argb| {
            let hct = Hct::from_int(argb);
            format!("HCT: ({:.1}, {:.1}, {:.1})", hct.hue(), hct.chroma(), hct.tone())
        };

        self.mismatches.push(format!(
            "[{scheme: <10}] {role: <25} | Exp: [{}] | Got: [{}] | ΔE: {d:.2}",
            fmt_hct(expected),
            fmt_hct(actual)
        ));

        // Increment counts separately
        *self.role_counts.entry(role.to_string()).or_insert(0) += 1;
        *self.scheme_counts.entry(scheme.to_string()).or_insert(0) += 1;
    }

    fn finalize(&self) -> Result<()> {
        if self.mismatches.is_empty() {
            println!("✅ All {} checks passed successfully.", self.total_tested);
            return Ok(());
        }

        let total_errs = self.mismatches.len();
        println!("\n⛔ FOUND {} MISMATCHES", total_errs);

        // 1. Random Sample
        let mut rng = rand::rng();
        let sample_size = 10.min(total_errs);
        println!("\n--- Random Sample ---");
        for m in self.mismatches.sample(&mut rng, sample_size) {
            println!("{m}");
        }

        // 2. Breakdown by Role
        println!("\n--- Top Failing Roles ---");
        let mut roles: Vec<_> = self.role_counts.iter().collect();
        roles.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in roles.iter().take(10) {
            let pct = (**count as f64 / total_errs as f64) * 100.0;
            println!("{name: <25}: {count: <5} ({pct:>5.1}% of all errors)");
        }

        // 3. Breakdown by Scheme
        println!("\n--- Top Failing Schemes ---");
        let mut schemes: Vec<_> = self.scheme_counts.iter().collect();
        schemes.sort_by(|a, b| b.1.cmp(a.1));
        for (name, count) in schemes.iter().take(10) {
            let pct = (**count as f64 / total_errs as f64) * 100.0;
            println!("{name: <25}: {count: <5} ({pct:>5.1}% of all errors)");
        }

        // 4. Global Stats
        println!("\n--- Error Magnitude (Cam16 ΔE) ---");
        println!("Mean:   {:.4}", (&self.distances).mean());
        println!("StdDev: {:.4}", (&self.distances).std_dev());

        Err(eyre!("Integration test failed: {total_errs} mismatches"))
    }
}

fn run_reference_test(path: &str, spec: SpecVersion, filter_role: Option<&str>) -> Result<()> {
    let content = fs::read_to_string(path).wrap_err("Failed to read reference file")?;
    let entries: Vec<ReferenceEntry> = serde_json::from_str(&content)?;
    let mdc = MaterialDynamicColors::new_with_spec(spec);
    let mut tracker = ValidationTracker::default();

    for entry in entries {
        let scheme = entry.to_dynamic_scheme()?;

        for getter in mdc.all_dynamic_colors() {
            let Some(dc) = getter() else { continue };

            if let Some(target) = filter_role {
                if dc.name != target {
                    continue;
                }
            }

            let actual = dc.get_argb(&scheme);
            let expected_hex = entry.roles.get(&dc.name).ok_or_else(|| {
                eyre!("Role {} missing in reference for {}", dc.name, entry.scheme)
            })?;

            let expected = entry.parse_color(expected_hex)?;
            tracker.total_tested += 1;

            if actual != expected {
                tracker.record_mismatch(&dc.name, &entry.scheme, expected, actual);
            }
        }
    }

    tracker.finalize()
}

#[test]
fn test_material_schemes_against_reference() -> Result<()> {
    run_reference_test(
        "tests/assets/json/reference_schemes_large.json",
        SpecVersion::Spec2026,
        None,
    )
}

#[test]
fn test_single_failing_color() -> Result<()> {
    run_reference_test(
        "tests/assets/json/reference_schemes_single.json",
        SpecVersion::Spec2026,
        Some("on_primary_container"),
    )
}
