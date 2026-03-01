use crate::contrast::contrast_utils::Contrast;
use crate::hct::Hct;
use crate::utils::color_utils::Argb;

/// Returns the contrast ratio of two colors.
#[must_use]
pub fn get_contrast_ratio(color1: Argb, color2: Argb) -> f64 {
    let t1 = color1.lstar();
    let t2 = color2.lstar();
    Contrast::ratio_of_tones(t1, t2)
}

/// Returns a lighter color with tone >= `tone` that meets `ratio`, or `None` if impossible.
#[must_use]
pub fn lighter_tone(color: Argb, ratio: f64) -> Option<Argb> {
    let hct = Hct::from_argb(color);
    let new_tone = Contrast::lighter(hct.tone(), ratio)?;
    Some(Hct::new(hct.hue(), hct.chroma(), new_tone).to_argb())
}

/// Returns a darker color with tone >= `tone` that meets `ratio`, or `None` if impossible.
#[must_use]
pub fn darker_tone(color: Argb, ratio: f64) -> Option<Argb> {
    let hct = Hct::from_argb(color);
    let new_tone = Contrast::darker(hct.tone(), ratio)?;
    Some(Hct::new(hct.hue(), hct.chroma(), new_tone).to_argb())
}

/// Unsafe variant of `lighter_tone`: always returns a value in [0,100].
#[must_use]
pub fn lighter_tone_unsafe(color: Argb, ratio: f64) -> Argb {
    let hct = Hct::from_argb(color);
    let new_tone = Contrast::lighter_unsafe(hct.tone(), ratio);
    Hct::new(hct.hue(), hct.chroma(), new_tone).to_argb()
}

/// Unsafe variant of `darker_tone`: always returns a value in [0,100].
#[must_use]
pub fn darker_tone_unsafe(color: Argb, ratio: f64) -> Argb {
    let hct = Hct::from_argb(color);
    let new_tone = Contrast::darker_unsafe(hct.tone(), ratio);
    Hct::new(hct.hue(), hct.chroma(), new_tone).to_argb()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::color_utils::Argb;
    use color_eyre::Result;
    use std::f64;

    #[test]
    fn test_get_contrast_ratio_black_white() -> Result<()> {
        let black = Argb::from_hex("#000000")?;
        let white = Argb::from_hex("#FFFFFF")?;
        let ratio = get_contrast_ratio(black, white);
        assert!((ratio - 21.0).abs() < f64::EPSILON);

        Ok(())
    }

    #[test]
    fn test_lighter_and_darker_tone_wrappers() -> Result<()> {
        let mid_tone_blue = Argb::from_hex("#3e5d7f")?;
        let desired = 3.0;

        let lighter_blue = lighter_tone(mid_tone_blue, desired).expect("Lighter should be Some");
        let darker_blue = darker_tone(mid_tone_blue, desired).expect("Darker should be Some");

        // Ensure tones moved in the expected directions
        let orig_tone = mid_tone_blue.lstar();
        let lighter_tone = lighter_blue.lstar();
        let darker_tone = darker_blue.lstar();
        assert!(lighter_tone > orig_tone);
        assert!(darker_tone < orig_tone);

        // And that the contrast ratio between each result and the original meets or exceeds desired
        let ratio_ligher = get_contrast_ratio(lighter_blue, mid_tone_blue);
        let ratio_darker = get_contrast_ratio(darker_blue, mid_tone_blue);

        assert!(ratio_ligher >= desired);
        assert!(ratio_darker >= desired);

        Ok(())
    }

    #[test]
    fn test_unsafe_variants_bounds() -> Result<()> {
        let bright_blue = Argb::from_hex("#2089f9")?;
        let unsafe_lighter_blue = lighter_tone_unsafe(bright_blue, 20.0);
        let safe_lighter_blue = lighter_tone(bright_blue, 20.0);

        assert!(safe_lighter_blue.is_none());

        let lighter_blue_tone = unsafe_lighter_blue.lstar();
        dbg!(lighter_blue_tone, unsafe_lighter_blue);
        assert!((lighter_blue_tone - 100.0).abs() < f64::EPSILON);

        Ok(())
    }
}
