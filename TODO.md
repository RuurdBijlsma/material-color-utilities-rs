# TODO:

* ✅ port cache for get_hct from DynamicColor.kt to dynamic_color.rs (maakt 'm niet sneller)
* ✅ Make integration test succeed 100%
    * ✅ implement color_spec 2025 & 2026
    * ✅ find out why the _dim colors arent found while they *are* there in the kotlin version
* ✅ Test quantize (colors from image)
* make more ergonomic api on top of this? find out intended api first
    * ✅ theme_from_color("#ff0000", Variant::Vibrant, 1.0) seems ideal of met bon builder
    * ✅ theme_from_image(&img, Variant..., 1.0)
    * ✅ colors_from_image(...
    * ✅ check_contrast(colorA, colorB)
* ✅ papaya cache (geen improvement)
* ✅ impl display for Argb & debug
* ✅ Score::score kan beter met bon builder, vind defaults in Score.kt
* ✅ rename to_int to to_argb?
* ✅ alle scheme_*.rs files omzetten om bon builder te gebruiken
* ✅ waarom is color_specs.rs get( met een Box?
* ✅ into trait voor u32->argb maken en reverseo
* ✅ benchmark en vergelijken met kotlin code
* ✅ tonalpallette cache bug
* ✅ contrast_curve get functie met match statement doen
* ✅ from impls maken voor kleur conversions?
  * ✅ from/to_hex_string
  * ✅ publieke functies die een kleur nemen zorgen dat ze Into argb accepteren 
* ✅ unwrap/expect/panic/assert weghalen
