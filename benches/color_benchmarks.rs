use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use material_color_utilities::dynamic::color_spec::{Platform, SpecVersion};
use material_color_utilities::dynamic::dynamic_scheme::DynamicScheme;
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;
use material_color_utilities::dynamic::variant::Variant;
use material_color_utilities::hct::hct_color::Hct;
use material_color_utilities::palettes::tonal_palette::TonalPalette;
use material_color_utilities::scheme::scheme_tonal_spot::SchemeTonalSpot;
use material_color_utilities::utils::color_utils::Argb;
use std::hint::black_box;

/// Benchmark Scheme Generation
fn bench_scheme_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scheme Generation");
    let argb = Argb(0xFF4285F4);

    for variant in [Variant::TonalSpot, Variant::Vibrant, Variant::Cmf] {
        group.bench_with_input(
            BenchmarkId::new("New Scheme", format!("{:?}", variant)),
            &variant,
            |b, &_v| {
                b.iter(|| {
                    // Testing specific scheme constructors
                    SchemeTonalSpot::builder(argb, false, 0.0)
                });
            },
        );
    }
    group.finish();
}

/// Benchmark Color Resolution (Tokens)
fn bench_color_resolution(c: &mut Criterion) {
    let mut group = c.benchmark_group("Color Resolution");

    let hct = Hct::from_argb(Argb(0xFF4285F4));

    // Create manual schemes to test specific spec versions
    let create_scheme = |spec: SpecVersion| {
        DynamicScheme::new_with_platform_and_spec(
            hct,
            Variant::TonalSpot,
            false,
            0.0,
            Platform::Phone,
            spec,
            TonalPalette::from_hct(hct),
            TonalPalette::from_hct(hct),
            TonalPalette::from_hct(hct),
            TonalPalette::from_hct(hct),
            TonalPalette::from_hct(hct),
            TonalPalette::from_hct(hct),
        )
    };

    let scheme_2021 = create_scheme(SpecVersion::Spec2021);
    let scheme_2026 = create_scheme(SpecVersion::Spec2026);
    let mdc = MaterialDynamicColors::new();

    group.bench_function("Resolve Surface (Spec2026)", |b| {
        let color = mdc.surface();
        b.iter(|| color.get_argb(black_box(&scheme_2026)))
    });

    group.bench_function("Resolve Primary (Spec2026 - Complex)", |b| {
        let color = mdc.primary();
        b.iter(|| color.get_argb(black_box(&scheme_2026)))
    });

    group.bench_function("Resolve Primary (Spec2021 - Legacy)", |b| {
        let color = MaterialDynamicColors::new_with_spec(SpecVersion::Spec2021).primary();
        b.iter(|| color.get_argb(black_box(&scheme_2021)))
    });

    group.finish();
}

/// Benchmark Bulk Palette Resolution
fn bench_bulk_resolution(c: &mut Criterion) {
    let scheme = SchemeTonalSpot::builder(Argb(0xFF4285F4), false, 0.0).build();
    let mdc = MaterialDynamicColors::new();
    let all_colors = mdc.all_dynamic_colors();

    c.bench_function("Resolve Full Palette (59 tokens)", |b| {
        b.iter(|| {
            for getter in &all_colors {
                if let Some(color) = getter() {
                    black_box(color.get_argb(&scheme));
                }
            }
        })
    });
}

criterion_group!(
    benches,
    bench_scheme_generation,
    bench_color_resolution,
    bench_bulk_resolution
);
criterion_main!(benches);
