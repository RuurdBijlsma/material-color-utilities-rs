use crate::quantize::Quantizer;
use crate::quantize::QuantizerCelebi;
use crate::score::score_colors::Score;
use crate::utils::color_utils::Argb;

/// Extract prominent colors from an image using quantization + scoring.
#[bon::builder]
pub fn extract_image_colors(
    /// The source image. This is the starting positional parameter.
    #[builder(start_fn)]
    image: &image::DynamicImage,
    /// Max colors to pass to the quantizer. Defaults to `128`.
    #[builder(default = 128)]
    quantize_max_colors: usize,
    /// Desired number of final colors to return. Defaults to `4`.
    #[builder(default = 4)]
    desired_colors: usize,
) -> Vec<Argb> {
    // Convert image to RGB8 to have consistent pixel layout and ignore alpha.
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

#[cfg(all(test, feature = "image"))]
mod tests {
    use super::*;
    use color_eyre::eyre::Result;
    use std::fs;

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

            println!("File: {}", path.display());
            println!("colors: {colors:?}");
        }

        Ok(())
    }
}
