use crate::hct::Hct;
use crate::palettes::tonal_palette::TonalPalette;
use crate::utils::color_utils::Argb;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
// ------ Argb ---------

impl Serialize for Argb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Argb {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

// ------ HCT ---------

impl Serialize for Hct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the underlying color as a hex string
        serializer.serialize_str(&self.to_argb().to_hex())
    }
}

impl<'de> Deserialize<'de> for Hct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let argb = Argb::from_hex(&s).map_err(serde::de::Error::custom)?;
        Ok(Self::from_argb(argb))
    }
}

// ------ TonalPalette ---------

#[derive(Serialize, Deserialize)]
struct TonalPaletteSurrogate {
    hue: f64,
    chroma: f64,
    tones: BTreeMap<i32, Argb>,
}

impl Serialize for TonalPalette {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tones = BTreeMap::new();
        // Export the standard Material 3 tonal steps
        for t in (10..=90).step_by(10) {
            tones.insert(t, self.tone(t));
        }

        let surrogate = TonalPaletteSurrogate {
            hue: self.hue,
            chroma: self.chroma,
            tones,
        };
        surrogate.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TonalPalette {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let surrogate = TonalPaletteSurrogate::deserialize(deserializer)?;
        let palette = Self::from_hue_and_chroma(surrogate.hue, surrogate.chroma);

        Ok(palette)
    }
}

#[cfg(test)]
mod serde_tests {
    use crate::dynamic::variant::Variant;
    use crate::hct::Hct;
    use crate::utils::color_utils::Argb;
    use crate::{MaterializedTheme, get_theme_from_color};
    use color_eyre::eyre::Result;

    #[test]
    fn test_theme_serde_round_trip() -> Result<()> {
        // Create a theme
        let color = Argb::from_hex("#4285F4")?;
        let theme = get_theme_from_color(color).variant(Variant::Cmf).call();

        // Serialize to JSON
        let json_string = serde_json::to_string_pretty(&theme)?;
        println!("{json_string}");

        // Verify JSON content requirements:
        assert!(json_string.contains("#4285F4"));
        assert!(json_string.contains("\"tones\""));
        assert!(json_string.contains("\"10\""));
        assert!(json_string.contains("\"90\""));

        // Deserialize back to object
        let deserialized: MaterializedTheme = serde_json::from_str(&json_string)?;

        // Verify structural integrity
        assert_eq!(deserialized, theme);

        Ok(())
    }

    #[test]
    fn test_hct_hex_serde() -> Result<()> {
        let hct = Hct::from_argb(Argb::from_hex("#FF0000")?);

        let json = serde_json::to_string(&hct)?;
        assert_eq!(json, "\"#FF0000\"");

        let back: Hct = serde_json::from_str(&json)?;
        assert_eq!(back.to_argb(), hct.to_argb());

        Ok(())
    }
}
