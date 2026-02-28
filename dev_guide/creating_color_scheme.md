# Creating a Color Scheme

See [Dynamic Color Scheme](../concepts/dynamic_color_scheme.md) for a conceptual
overview.

## Getting colors for theming

To get color values for styling your UIs, the first step is to create a
`DynamicScheme`. The easiest way is to use one of the provided scheme variant
constructors. Each variant requires:

1.  A source color in HCT format (`Hct`).
1.  Whether the scheme is in dark mode (`is_dark: bool`).
1.  Contrast level (`contrast_level: f64`). The recommended levels are:
    -   `0.0` for default contrast.
    -   `0.5` for medium contrast.
    -   `1.0` for highest contrast.
    -   `-1.0` for reduced contrast.

The `DynamicScheme` struct does not store pre-computed ARGB values for each
color role. Instead, values are computed on demand. You can retrieve them either
via the convenience methods on `DynamicScheme` (e.g. `scheme.primary()`) or via
the lower-level `DynamicColor` API.

## Step-by-Step Instructions

### 1. Generating a scheme

#### Method 1 — Using a variant constructor

The recommended way to generate a scheme is to use a variant constructor such as
`SchemeTonalSpot::new`. The following example generates a tonal-spot scheme in
light mode with default contrast from an `Hct` source color.

```rust
use material_color_utilities::hct::Hct;
use material_color_utilities::scheme::SchemeTonalSpot;
use material_color_utilities::utils::color_utils::Argb;

let source_argb = Argb(0xFF6750A4); // a purple
let hct = Hct::from_argb(source_argb);
let scheme = SchemeTonalSpot::new(hct, false, 0.0); // light mode, default contrast
```

The available variants are:

*   `SchemeTonalSpot` — calm, sedated colors
*   `SchemeContent` — colors derived closely from the source color
*   `SchemeExpressive` — highly colorful, playful
*   `SchemeFidelity` — maximally faithful to source color
*   `SchemeFruitSalad` — multiple hues
*   `SchemeMonochrome` — grayscale
*   `SchemeNeutral` — near-neutral palette
*   `SchemeRainbow` — rainbow-like palette
*   `SchemeVibrant` — highly saturated

All are in the `material_color_utilities::scheme` module and share the same
`::new(source_color_hct: Hct, is_dark: bool, contrast_level: f64) -> DynamicScheme`
signature.

For advanced use, the `::new_with_platform_and_spec` variants allow specifying
a `Platform` and `SpecVersion`.

#### Method 2 — Building a scheme from explicit palettes

You can also construct a `DynamicScheme` directly, providing your own tonal palettes:

```rust
use material_color_utilities::dynamic::DynamicScheme;
use material_color_utilities::dynamic::variant::Variant;
use material_color_utilities::hct::Hct;
use material_color_utilities::palettes::tonal_palette::TonalPalette;
use material_color_utilities::utils::color_utils::Argb;

let source_hct = Hct::from_argb(Argb(0xFFEB0057));

let scheme = DynamicScheme::new(
    source_hct,
    Variant::Vibrant,
    /*is_dark=*/ false,
    /*contrast_level=*/ 0.0,
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFFEB0057))), // primary_palette
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFFF46B00))), // secondary_palette
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFF00AB46))), // tertiary_palette
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFF949494))), // neutral_palette
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFFBC8877))), // neutral_variant_palette
    TonalPalette::from_hct(Hct::from_argb(Argb(0xFFFF0000))), // error palette
);
```

### 2. Obtaining colors

Colors are returned as `Argb` values. The `DynamicScheme` struct exposes
convenience methods for every standard Material color role:

```rust
let primary: Argb = scheme.primary();
let on_primary: Argb = scheme.on_primary();
let primary_container: Argb = scheme.primary_container();
let secondary: Argb = scheme.secondary();
let surface: Argb = scheme.surface();
// … and so on for all roles
```

To obtain both the ARGB value and the HCT representation, use the lower-level
`DynamicColor` API via `MaterialDynamicColors`:

```rust
use material_color_utilities::dynamic::material_dynamic_colors::MaterialDynamicColors;

let mdc = MaterialDynamicColors::new();
let primary_argb = scheme.get_argb(&mdc.primary());
let primary_hct  = scheme.get_hct(&mdc.primary());
```

### 3. Switching between light and dark

To create the opposite-mode counterpart of an existing scheme without
reconstructing it from scratch, use `DynamicScheme::from_scheme`:

```rust
let light_scheme = SchemeTonalSpot::new(hct, false, 0.0);
let dark_scheme  = DynamicScheme::from_scheme(&light_scheme, true);
```

To also change the contrast level at the same time:

```rust
let high_contrast_dark = DynamicScheme::from_scheme_with_contrast(&light_scheme, true, 1.0);
```
