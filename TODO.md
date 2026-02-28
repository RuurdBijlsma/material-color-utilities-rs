# TODO:

* ✅ port cache for get_hct from DynamicColor.kt to dynamic_color.rs (maakt 'm niet sneller)
* ✅ Make integration test succeed 100%
    * ✅ implement color_spec 2025 & 2026
    * ✅ find out why the _dim colors arent found while they *are* there in the kotlin version
* ✅ Test quantize (colors from image)
* make more ergonomic api on top of this? find out intended api first
    * theme_from_color("#ff0000", Variant::Vibrant, 1.0) seems ideal of met bon builder
    * theme_from_image(&img, Variant..., 1.0)
    * colors_from_image(...
    * check_contrast(colorA, colorB)
    * misschien voor alles wat een kleur accepteert een into trait accepteren ofzo (into Argb) zodat je het volgende kan
      sturen
        * "#ff0000"
        * Argb(0xFFFF0000)
        * Hct(...
* code improvements
* ✅ papaya cache (geen improvement)
* ✅ maak dit workspace & geef de "pure" port zijn eigen crate, zet abstracties in andere crate om het apart te houden (
  dan moet ik 2 crates publishen dat vind ik stom)