/*
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/// Utility methods for string representations of colors.
pub struct StringUtils;

impl StringUtils {
    /// Hex string representing color, ex. #ff0000 for red.
    ///
    /// # Arguments
    ///
    /// * `argb` - ARGB representation of a color.
    pub fn hex_from_argb(argb: u32) -> String {
        let red = (argb >> 16) & 255;
        let green = (argb >> 8) & 255;
        let blue = argb & 255;
        format!("#{:02x}{:02x}{:02x}", red, green, blue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_from_argb() {
        assert_eq!(StringUtils::hex_from_argb(0xFFFF0000), "#ff0000");
        assert_eq!(StringUtils::hex_from_argb(0xFF00FF00), "#00ff00");
        assert_eq!(StringUtils::hex_from_argb(0xFF0000FF), "#0000ff");
        assert_eq!(StringUtils::hex_from_argb(0xFFFFFFFF), "#ffffff");
        assert_eq!(StringUtils::hex_from_argb(0xFF000000), "#000000");
    }
}
