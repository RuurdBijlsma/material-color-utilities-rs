use crate::utils::color_utils::Argb;

/// Utility methods for string representations of colors.
pub struct StringUtils;

impl StringUtils {
    /// Hex string representing color, ex. #ff0000 for red.
    ///
    /// # Arguments
    ///
    /// * `argb` - ARGB representation of a color.
    #[must_use]
    pub fn hex_from_argb(argb: Argb) -> String {
        let argb = argb.0;
        let red = (argb >> 16) & 255;
        let green = (argb >> 8) & 255;
        let blue = argb & 255;
        format!("#{red:02x}{green:02x}{blue:02x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_from_argb() {
        assert_eq!(StringUtils::hex_from_argb(Argb(0xFFFF0000)), "#ff0000");
        assert_eq!(StringUtils::hex_from_argb(Argb(0xFF00FF00)), "#00ff00");
        assert_eq!(StringUtils::hex_from_argb(Argb(0xFF0000FF)), "#0000ff");
        assert_eq!(StringUtils::hex_from_argb(Argb(0xFFFFFFFF)), "#ffffff");
        assert_eq!(StringUtils::hex_from_argb(Argb(0xFF000000)), "#000000");
    }
}
