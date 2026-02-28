# Refining Contrast

In this guide, you will learn how to manually refine color contrast using the
MCU contrast library. For optimal color contrast, we recommend using the
`DynamicColor` and `DynamicScheme` features in your production code.

See [Contrast for Accessibility](../concepts/contrast_for_accessibility.md) for
a conceptual overview.

All functions described here live in `material_color_utilities::contrast::Contrast`.

## Calculating contrast ratio

To measure the contrast of two colors, use `Contrast::ratio_of_tones` on the
tones (`L*`) of the two colors.

The tone of an `Hct` color is `hct.tone()`. The tone of an `Argb` color can be
obtained via `argb.lstar()`.

```rust
use material_color_utilities::contrast::Contrast;
use material_color_utilities::hct::Hct;
use material_color_utilities::utils::color_utils::Argb;

// From HCT colors:
let contrast_ratio = Contrast::ratio_of_tones(hct1.tone(), hct2.tone());

// From ARGB colors:
let tone1 = argb1.lstar();
let tone2 = argb2.lstar();
let contrast_ratio = Contrast::ratio_of_tones(tone1, tone2);
```

Useful contrast ratio constants are available on the `Contrast` struct:

| Constant | Value | Meaning |
|---|---|---|
| `Contrast::RATIO_MIN` | 1.0 | Same color |
| `Contrast::RATIO_30` | 3.0 | WCAG AA non-text |
| `Contrast::RATIO_45` | 4.5 | WCAG AA text |
| `Contrast::RATIO_70` | 7.0 | WCAG AAA text |
| `Contrast::RATIO_MAX` | 21.0 | Black on white |

## Obtaining well-contrasting tones

`Contrast::darker` and `Contrast::lighter` return an `Option<f64>`, yielding
`None` when the requested contrast ratio cannot be achieved.

`Contrast::darker_unsafe` and `Contrast::lighter_unsafe` always return a value
in `[0, 100]` — clamping to black (0) or white (100) when the ratio cannot be
met — but do **not** guarantee the contrast ratio will be reached.

```rust
use material_color_utilities::contrast::Contrast;
use material_color_utilities::utils::color_utils::Argb;

let original = Argb(0xFF00AA00).lstar(); // ≈ 60.56

// Tones that contrast at 3:1 with original:
let darker  = Contrast::darker(original, 3.0);        // Some(≈29.63)
let lighter = Contrast::lighter(original, 3.0);       // Some(≈98.93)

// If the ratio cannot be reached, None is returned:
let darker7  = Contrast::darker(original, 7.0);       // None
let lighter7 = Contrast::lighter(original, 7.0);      // None

// Unsafe variants always return a value:
let darker_unsafe  = Contrast::darker_unsafe(original, 7.0);   // 0.0
let lighter_unsafe = Contrast::lighter_unsafe(original, 7.0);  // 100.0
```

To convert a tone back to an `Argb` grayscale or apply it to a hue+chroma,
use `Argb::from_lstar(tone)` or `Hct::from(hue, chroma, tone).to_argb()`.
