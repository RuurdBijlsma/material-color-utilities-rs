use rust::hct::hct::Hct;
use rust::dynamiccolor::material_dynamic_colors::MaterialDynamicColors;
use rust::scheme::scheme_tonal_spot::SchemeTonalSpot;
use rust::scheme::scheme_content::SchemeContent;
use rust::scheme::scheme_vibrant::SchemeVibrant;
use rust::scheme::scheme_monochrome::SchemeMonochrome;
use rust::scheme::scheme_expressive::SchemeExpressive;
use rust::utils::color_utils::Argb;
use rust::dynamiccolor::color_spec::SpecVersion;

macro_rules! test_scheme {
    (
        name: $name:ident,
        scheme: $scheme_type:ident,
        source: $source_color:expr,
        is_dark: $is_dark:expr,
        expected: {
            $( $method:ident : $expected:expr ),* $(,)?
        }
    ) => {
        #[test]
        fn $name() {
            let hct = Hct::from_int(Argb($source_color));
            let scheme = $scheme_type::new(hct, $is_dark, 0.0);
            let colors = MaterialDynamicColors::new_with_spec(SpecVersion::Spec2021);

            $(
                let actual = colors.$method().get_argb(&scheme).0;
                assert_eq!(
                    actual,
                    $expected,
                    "{}(), expected {:#010X} but got {:#010X}",
                    stringify!($method),
                    $expected,
                    actual
                );
            )*
        }
    };
}

// ----------------------------------------------------------------------------
// Tonal Spot Tests
// ----------------------------------------------------------------------------
test_scheme!(
    name: test_tonal_spot_light,
    scheme: SchemeTonalSpot,
    source: 0xFF4285F4, // Google Blue
    is_dark: false,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

test_scheme!(
    name: test_tonal_spot_dark,
    scheme: SchemeTonalSpot,
    source: 0xFF4285F4,
    is_dark: true,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

// ----------------------------------------------------------------------------
// Content Tests
// ----------------------------------------------------------------------------
test_scheme!(
    name: test_content_light,
    scheme: SchemeContent,
    source: 0xFFEA4335, // Google Red
    is_dark: false,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

test_scheme!(
    name: test_content_dark,
    scheme: SchemeContent,
    source: 0xFFEA4335,
    is_dark: true,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

// ----------------------------------------------------------------------------
// Vibrant Tests
// ----------------------------------------------------------------------------
test_scheme!(
    name: test_vibrant_light,
    scheme: SchemeVibrant,
    source: 0xFFFBBC04, // Google Yellow
    is_dark: false,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

test_scheme!(
    name: test_vibrant_dark,
    scheme: SchemeVibrant,
    source: 0xFFFBBC04,
    is_dark: true,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

// ----------------------------------------------------------------------------
// Monochrome Tests
// ----------------------------------------------------------------------------
test_scheme!(
    name: test_monochrome_light,
    scheme: SchemeMonochrome,
    source: 0xFF34A853, // Google Green
    is_dark: false,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

test_scheme!(
    name: test_monochrome_dark,
    scheme: SchemeMonochrome,
    source: 0xFF34A853,
    is_dark: true,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

// ----------------------------------------------------------------------------
// Expressive Tests
// ----------------------------------------------------------------------------
test_scheme!(
    name: test_expressive_light,
    scheme: SchemeExpressive,
    source: 0xFF6200EE, // Material Purple
    is_dark: false,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);

test_scheme!(
    name: test_expressive_dark,
    scheme: SchemeExpressive,
    source: 0xFF6200EE,
    is_dark: true,
    expected: {
        primary: 0x00000000,
        on_primary: 0x00000000,
        primary_container: 0x00000000,
        on_primary_container: 0x00000000,
        inverse_primary: 0x00000000,

        secondary: 0x00000000,
        on_secondary: 0x00000000,
        secondary_container: 0x00000000,
        on_secondary_container: 0x00000000,

        tertiary: 0x00000000,
        on_tertiary: 0x00000000,
        tertiary_container: 0x00000000,
        on_tertiary_container: 0x00000000,

        error: 0x00000000,
        on_error: 0x00000000,
        error_container: 0x00000000,
        on_error_container: 0x00000000,

        background: 0x00000000,
        on_background: 0x00000000,
        
        surface: 0x00000000,
        on_surface: 0x00000000,
        surface_variant: 0x00000000,
        on_surface_variant: 0x00000000,
        inverse_surface: 0x00000000,
        inverse_on_surface: 0x00000000,
        
        surface_dim: 0x00000000,
        surface_bright: 0x00000000,
        surface_container_lowest: 0x00000000,
        surface_container_low: 0x00000000,
        surface_container: 0x00000000,
        surface_container_high: 0x00000000,
        surface_container_highest: 0x00000000,

        outline: 0x00000000,
        outline_variant: 0x00000000,
        shadow: 0x00000000,
        scrim: 0x00000000,
        surface_tint: 0x00000000,
    }
);
