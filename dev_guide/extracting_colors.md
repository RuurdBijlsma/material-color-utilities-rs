# Extracting colors from an image

See [Color Extraction](../concepts/color_extraction.md) for a conceptual
overview.

## Step 1 — Image to Pixels

The first step is to convert an image into **a slice of `Argb` values**.
Prior to quantization, resize the image to 128 × 128 pixels for faster
processing.

MCU does not provide image loading or resizing itself. Use a crate such as
[`image`](https://crates.io/crates/image) from the ecosystem:

```rust
use material_color_utilities::utils::color_utils::Argb;

fn image_to_pixels(img: &image::DynamicImage) -> Vec<Argb> {
    let img = img.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);
    let rgb = img.to_rgba8();
    rgb.pixels()
        .map(|p| Argb::from_rgb(p[0], p[1], p[2]))
        .collect()
}
```

## Step 2 — Pixels to Prominent Colors

Once you have the pixel slice, pass it to `QuantizerCelebi` to obtain a
`QuantizerResult`, which maps each representative color to its pixel count.

```rust
use material_color_utilities::quantize::{QuantizerCelebi, Quantizer};

let mut quantizer = QuantizerCelebi::new();
let result = quantizer.quantize(&pixels, /*max_colors=*/ 128);
// result.color_to_count: IndexMap<Argb, u32>
```

`max_colors` is an upper bound on the number of colors returned. 128 is a
reasonable default that gives good quality results.

## Step 3 — Prominent Colors to Source Colors

Use `Score::score` to rank the quantized colors by suitability as a source
color for a dynamic color scheme. The result is sorted from most to least
suitable.

```rust
use material_color_utilities::score::Score;

let source_colors: Vec<Argb> = Score::score(&result.color_to_count);
// source_colors[0] is the best candidate for a scheme source color.
```

Additional entry points:

| Function | Description |
|---|---|
| `Score::score(map)` | Up to 4 results, Google Blue as fallback |
| `Score::score_desired(map, n)` | Up to `n` results |
| `Score::score_fallback(map, n, fallback)` | Up to `n` results with a custom fallback color |
| `Score::score_with_options(map, n, fallback, filter)` | Full control |

## Putting it all together

```rust
use material_color_utilities::quantize::{QuantizerCelebi, Quantizer};
use material_color_utilities::score::Score;
use material_color_utilities::hct::Hct;
use material_color_utilities::scheme::SchemeTonalSpot;

let mut quantizer = QuantizerCelebi::new();
let result = quantizer.quantize(&pixels, 128);

let source_colors = Score::score(&result.color_to_count);
let source_hct = Hct::from_int(source_colors[0]);

let scheme = SchemeTonalSpot::new(source_hct, /*is_dark=*/ false, /*contrast_level=*/ 0.0);
println!("Primary: {:?}", scheme.primary());
```
