use crate::hct::{Cam16, Hct};
use crate::utils::color_utils::{Argb, Lab, Xyz};

/// sRGB ⇌ XYZ
impl From<Argb> for Xyz {
    fn from(argb: Argb) -> Self {
        argb.to_xyz()
    }
}

impl From<Xyz> for Argb {
    fn from(xyz: Xyz) -> Self {
        Self::from_xyz(xyz)
    }
}

/// sRGB ⇌ L*a*b*
impl From<Argb> for Lab {
    fn from(argb: Argb) -> Self {
        argb.to_lab()
    }
}

impl From<Lab> for Argb {
    fn from(lab: Lab) -> Self {
        Self::from_lab(lab)
    }
}

/// linRGB → sRGB
///
/// Converts a linear-RGB triple `[r, g, b]` (values in 0–100) to `Argb`.
impl From<[f64; 3]> for Argb {
    fn from(linrgb: [f64; 3]) -> Self {
        Self::from_linrgb(linrgb)
    }
}

/// sRGB ⇌ HCT
impl From<Argb> for Hct {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}

impl From<Hct> for Argb {
    fn from(hct: Hct) -> Self {
        hct.to_argb()
    }
}

/// sRGB ⇌ CAM16 (default viewing conditions)
impl From<Argb> for Cam16 {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}

impl From<Cam16> for Argb {
    fn from(cam16: Cam16) -> Self {
        cam16.to_argb()
    }
}
