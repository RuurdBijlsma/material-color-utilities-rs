use crate::hct::{Cam16, Hct};
use crate::utils::color_utils::{Argb, Lab, Xyz};

// --- Argb Conversions ---

impl From<Xyz> for Argb {
    fn from(xyz: Xyz) -> Self {
        Self::from_xyz(xyz)
    }
}

impl From<Lab> for Argb {
    fn from(lab: Lab) -> Self {
        Self::from_lab(lab)
    }
}

impl From<Hct> for Argb {
    fn from(hct: Hct) -> Self {
        hct.to_argb()
    }
}

impl From<Cam16> for Argb {
    fn from(cam16: Cam16) -> Self {
        cam16.to_argb()
    }
}

/// linRGB [0-100] → sRGB
impl From<[f64; 3]> for Argb {
    fn from(linrgb: [f64; 3]) -> Self {
        Self::from_linrgb(linrgb)
    }
}

impl From<u32> for Argb {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Argb> for u32 {
    fn from(argb: Argb) -> Self {
        argb.0
    }
}

// --- Hct Conversions ---

impl From<Argb> for Hct {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}

impl From<Xyz> for Hct {
    fn from(xyz: Xyz) -> Self {
        Self::from_argb(Argb::from(xyz))
    }
}

impl From<Lab> for Hct {
    fn from(lab: Lab) -> Self {
        Self::from_argb(Argb::from(lab))
    }
}

impl From<Cam16> for Hct {
    fn from(cam16: Cam16) -> Self {
        Self::from_argb(Argb::from(cam16))
    }
}

impl From<u32> for Hct {
    fn from(value: u32) -> Self {
        Self::from_argb(Argb(value))
    }
}

impl From<Hct> for u32 {
    fn from(hct: Hct) -> Self {
        hct.to_argb().0
    }
}

// --- Other Conversions (ARGB as source) ---

impl From<Argb> for Xyz {
    fn from(argb: Argb) -> Self {
        argb.to_xyz()
    }
}

impl From<Argb> for Lab {
    fn from(argb: Argb) -> Self {
        argb.to_lab()
    }
}

impl From<Argb> for Cam16 {
    fn from(argb: Argb) -> Self {
        Self::from_argb(argb)
    }
}
