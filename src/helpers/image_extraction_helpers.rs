use crate::dynamic::color_spec::{Platform, SpecVersion};
use crate::dynamic::variant::Variant;
use crate::helpers::error::ThemeGenerationError;
use crate::quantize::Quantizer;
use crate::quantize::QuantizerCelebi;
use crate::score::score_colors::Score;
use crate::utils::color_utils::Argb;
use crate::{MaterializedTheme, theme_from_color};
use image::DynamicImage;
#[cfg(feature = "rayon")]
use rayon::iter::ParallelIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::IntoParallelRefIterator;

/// Extract prominent colors from an image using quantization + scoring.
#[bon::builder]
pub fn extract_image_colors(
    /// The source image.
    #[builder(start_fn)]
    image: &DynamicImage,
    /// Max colors to pass to the quantizer.
    #[builder(default = 128)]
    quantize_max_colors: usize,
    /// Desired number of final colors to return.
    #[builder(default = 4)]
    desired_colors: usize,
) -> Vec<Argb> {
    let pixels: Vec<Argb> = image
        .to_rgb8()
        .pixels()
        .map(|p| {
            let [r, g, b] = p.0;
            Argb::from_rgb(r, g, b)
        })
        .collect();
    let mut celebi = QuantizerCelebi::new();
    let result = celebi.quantize(&pixels, quantize_max_colors);

    Score::score(&result.color_to_count)
        .desired_count(desired_colors)
        .call()
}

/// Generate multiple themes from a source image.
#[bon::builder]
pub fn themes_from_image(
    /// The source image.
    #[builder(start_fn)]
    image: &DynamicImage,
    /// Max colors to pass to the quantizer.
    #[builder(default = 128)]
    quantize_max_colors: usize,
    /// Desired number of themes to generate from this image.
    #[builder(default = 4)]
    desired_theme_count: usize,
    /// Which variant to use
    #[builder(default = Variant::Vibrant)]
    variant: Variant,
    /// Contrast level.
    ///  - `0.0` for default contrast.
    ///  - `0.5` for medium contrast.
    ///  - `1.0` for highest contrast.
    ///  - `-1.0` for reduced contrast.
    #[builder(default = 0.0)]
    contrast_level: f64,
    /// `SpecVersion` tracks which version of the Material Design dynamic color spec the algorithms are following.
    #[builder(default = SpecVersion::Spec2026)]
    spec_version: SpecVersion,
    /// What platform to optimize colors for.
    #[builder(default = Platform::Phone)]
    platform: Platform,
) -> Vec<MaterializedTheme> {
    let colors = extract_image_colors(image)
        .desired_colors(desired_theme_count)
        .quantize_max_colors(quantize_max_colors)
        .call();

    #[cfg(feature = "rayon")]
    let iter = colors.par_iter();
    #[cfg(not(feature = "rayon"))]
    let iter = colors.iter();

    iter.map(|c| {
        theme_from_color(*c)
            .spec_version(spec_version)
            .platform(platform)
            .contrast_level(contrast_level)
            .variant(variant)
            .call()
    })
    .collect()
}

/// Generate single theme from a source image.
#[bon::builder]
pub fn theme_from_image(
    /// The source image.
    #[builder(start_fn)]
    image: &DynamicImage,
    /// Max colors to pass to the quantizer.
    #[builder(default = 128)]
    quantize_max_colors: usize,
    /// Which variant to use
    #[builder(default = Variant::Vibrant)]
    variant: Variant,
    /// Contrast level.
    ///  - `0.0` for default contrast.
    ///  - `0.5` for medium contrast.
    ///  - `1.0` for highest contrast.
    ///  - `-1.0` for reduced contrast.
    #[builder(default = 0.0)]
    contrast_level: f64,
    /// `SpecVersion` tracks which version of the Material Design dynamic color spec the algorithms are following.
    #[builder(default = SpecVersion::Spec2026)]
    spec_version: SpecVersion,
    /// What platform to optimize colors for.
    #[builder(default = Platform::Phone)]
    platform: Platform,
) -> Result<MaterializedTheme, ThemeGenerationError> {
    let colors = extract_image_colors(image)
        .desired_colors(1)
        .quantize_max_colors(quantize_max_colors)
        .call();

    let Some(color) = colors.first() else {
        return Err(ThemeGenerationError::CouldNotExtractColorFromImage);
    };

    Ok(theme_from_color(*color)
        .spec_version(spec_version)
        .platform(platform)
        .contrast_level(contrast_level)
        .variant(variant)
        .call())
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::eyre::{Context, Result};
    use std::fs;
    use std::path::Path;

    const TEST_IMAGE_PATH: &str = "tests/assets/img/river.png";

    /// Helper to load the test image or skip if missing
    fn load_test_image() -> Result<DynamicImage> {
        let path = Path::new(TEST_IMAGE_PATH);
        image::open(path).wrap_err("Failed to open test image")
    }

    #[test]
    fn test_theme_from_image_with_river() -> Result<()> {
        let theme_result = theme_from_image(&load_test_image()?)
            .variant(Variant::Vibrant)
            .call();

        // Check that the generation succeeded
        assert!(
            theme_result.is_ok(),
            "Theme generation failed: {:?}",
            theme_result.err()
        );

        let _theme = theme_result?;

        Ok(())
    }

    #[test]
    fn test_themes_from_image_with_river() -> Result<()> {
        let img = load_test_image()?;
        let desired_count = 3;

        let themes = themes_from_image(&img)
            .desired_theme_count(desired_count)
            .variant(Variant::Expressive)
            .platform(Platform::Watch)
            .call();

        assert_eq!(
            themes.len(),
            desired_count,
            "Should not exceed the desired count"
        );

        Ok(())
    }

    #[test]
    fn integration_extract_colors_print() -> Result<()> {
        let dir = "tests/assets/img";

        for entry in fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            let img = image::open(&path)?;

            let colors = extract_image_colors(&img)
                .quantize_max_colors(128)
                .desired_colors(4)
                .call();

            assert_eq!(colors.len(), 4);
        }

        Ok(())
    }
}
