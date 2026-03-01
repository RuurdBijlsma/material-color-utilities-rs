use crate::hct::cam16::Cam16;
use crate::hct::hct_solver::HctSolver;
use crate::hct::viewing_conditions::ViewingConditions;
use crate::utils::color_utils::{Argb, ColorUtils};
use std::fmt;

/// HCT, hue, chroma, and tone. A color system that provides a perceptually accurate color
/// measurement system that can also accurately render what colors will appear as in different
/// lighting environments.
///
/// A color system built using CAM16 hue and chroma, and L* from L*a*b*.
///
/// Using L* creates a link between the color system, contrast, and thus accessibility. Contrast
/// ratio depends on relative luminance, or Y in the XYZ color space. L*, or perceptual luminance can
/// be calculated from Y.
///
/// Unlike Y, L* is linear to human perception, allowing trivial creation of accurate color tones.
///
/// Unlike contrast ratio, measuring contrast in L* is linear, and simple to calculate. A difference
/// of 40 in HCT tone guarantees a contrast ratio >= 3.0, and a difference of 50 guarantees a
/// contrast ratio >= 4.5.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hct {
    hue: f64,
    chroma: f64,
    tone: f64,
    argb: Argb,
}

impl Eq for Hct {}

impl std::hash::Hash for Hct {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.argb.hash(state);
    }
}

impl Hct {
    fn new_internal(argb: Argb) -> Self {
        let cam = Cam16::from_argb(argb);
        Self {
            hue: cam.hue,
            chroma: cam.chroma,
            tone: argb.lstar(),
            argb,
        }
    }

    /// Create an HCT color from hue, chroma, and tone.
    ///
    /// # Arguments
    ///
    /// * `hue`: 0 <= hue < 360; invalid values are corrected.
    /// * `chroma`: 0 <= chroma < ?; Informally, colorfulness. The color returned may be lower than
    ///   the requested chroma. Chroma has a different maximum for any given hue and tone.
    /// * `tone`: 0 <= tone <= 100; invalid values are corrected.
    ///
    /// # Returns
    ///
    /// HCT representation of a color in default viewing conditions.
    #[must_use]
    pub fn new(hue: f64, chroma: f64, tone: f64) -> Self {
        let argb = HctSolver::solve_to_argb(hue, chroma, tone);
        Self::new_internal(argb)
    }

    /// Create an HCT color from a color.
    ///
    /// # Arguments
    ///
    /// * `argb`: ARGB representation of a color.
    ///
    /// # Returns
    ///
    /// HCT representation of a color in default viewing conditions.
    #[must_use]
    pub fn from_argb(argb: Argb) -> Self {
        Self::new_internal(argb)
    }

    #[must_use]
    pub const fn hue(&self) -> f64 {
        self.hue
    }

    #[must_use]
    pub const fn chroma(&self) -> f64 {
        self.chroma
    }

    #[must_use]
    pub const fn tone(&self) -> f64 {
        self.tone
    }

    #[must_use]
    pub const fn to_argb(&self) -> Argb {
        self.argb
    }

    /// Set the hue of this color. Chroma may decrease because chroma has a different maximum for any
    /// given hue and tone.
    ///
    /// # Arguments
    ///
    /// * `new_hue`: 0 <= `new_hue` < 360; invalid values are corrected.
    pub fn set_hue(&mut self, new_hue: f64) {
        self.set_internal_state(HctSolver::solve_to_argb(new_hue, self.chroma, self.tone));
    }

    /// Set the chroma of this color. Chroma may decrease because chroma has a different maximum for
    /// any given hue and tone.
    ///
    /// # Arguments
    ///
    /// * `new_chroma`: 0 <= `new_chroma` < ?
    pub fn set_chroma(&mut self, new_chroma: f64) {
        self.set_internal_state(HctSolver::solve_to_argb(self.hue, new_chroma, self.tone));
    }

    /// Set the tone of this color. Chroma may decrease because chroma has a different maximum for any
    /// given hue and tone.
    ///
    /// # Arguments
    ///
    /// * `new_tone`: 0 <= `new_tone` <= 100; invalid values are corrected.
    pub fn set_tone(&mut self, new_tone: f64) {
        self.set_internal_state(HctSolver::solve_to_argb(self.hue, self.chroma, new_tone));
    }

    fn set_internal_state(&mut self, argb: Argb) {
        self.argb = argb;
        let cam = Cam16::from_argb(argb);
        self.hue = cam.hue;
        self.chroma = cam.chroma;
        self.tone = argb.lstar();
    }

    /// Translate a color into different `ViewingConditions`.
    ///
    /// Colors change appearance. They look different with lights on versus off, the same color, as in
    /// hex code, on white looks different when on black. This is called color relativity, most
    /// famously explicated by Josef Albers in Interaction of Color.
    ///
    /// In color science, color appearance models can account for this and calculate the appearance of
    /// a color in different settings. HCT is based on CAM16, a color appearance model, and uses it to
    /// make these calculations.
    ///
    /// See `ViewingConditions::make` for parameters affecting color appearance.
    #[must_use]
    pub fn in_viewing_conditions(&self, vc: &ViewingConditions) -> Self {
        // 1. Use CAM16 to find XYZ coordinates of color in specified VC.
        let cam16 = Cam16::from_argb(self.argb);
        let viewed_in_vc = cam16.xyz_in_viewing_conditions(vc);

        // 2. Create CAM16 of those XYZ coordinates in default VC.
        let recast_in_vc = Cam16::from_xyz_in_viewing_conditions(
            viewed_in_vc.x,
            viewed_in_vc.y,
            viewed_in_vc.z,
            &ViewingConditions::default(),
        );

        // 3. Create HCT from:
        // - CAM16 using default VC with XYZ coordinates in specified VC.
        // - L* converted from Y in XYZ coordinates in specified VC.
        Self::new(
            recast_in_vc.hue,
            recast_in_vc.chroma,
            ColorUtils::lstar_from_y(viewed_in_vc.y),
        )
    }

    #[must_use]
    pub fn is_blue(hue: f64) -> bool {
        (250.0..270.0).contains(&hue)
    }

    #[must_use]
    pub fn is_yellow(hue: f64) -> bool {
        (105.0..125.0).contains(&hue)
    }

    #[must_use]
    pub fn is_cyan(hue: f64) -> bool {
        (170.0..207.0).contains(&hue)
    }
}

impl fmt::Display for Hct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HCT({}, {}, {})",
            self.hue.round(),
            self.chroma.round(),
            self.tone.round()
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hct_red() {
        let hct = Hct::new(0.0, 50.0, 50.0);
        // We don't check exact values because rounding and floating point might vary slightly
        // but it should be close to what we requested.
        assert!((hct.hue() - 0.0).abs() < 1.0 || (hct.hue() - 360.0).abs() < 1.0);
        assert!(hct.chroma() > 0.0);
        assert!((hct.tone() - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_hct_from_argb() {
        let argb = Argb(0xFF00FF00); // Green
        let hct = Hct::from_argb(argb);
        assert_eq!(hct.to_argb(), argb);
        assert!(hct.chroma() > 0.0);
    }

    #[test]
    fn test_hct_setters() {
        let mut hct = Hct::new(120.0, 60.0, 50.0);
        hct.set_hue(200.0);
        assert!((hct.hue() - 200.0).abs() < 1.0);

        hct.set_chroma(30.0);
        assert!((hct.chroma() - 30.0).abs() < 1.0);

        hct.set_tone(80.0);
        assert!((hct.tone() - 80.0).abs() < 1.0);
    }

    #[test]
    fn test_hct_in_viewing_conditions() {
        let hct = Hct::new(0.0, 50.0, 50.0);
        let vc = ViewingConditions::default();
        let hct_vc = hct.in_viewing_conditions(&vc);
        // In default conditions, it should stay the same
        assert!((hct.hue() - hct_vc.hue()).abs() < 1.0);
        assert!((hct.chroma() - hct_vc.chroma()).abs() < 1.0);
        assert!((hct.tone() - hct_vc.tone()).abs() < 1.0);
    }

    #[test]
    fn test_hct_hue_checks() {
        assert!(Hct::is_blue(260.0));
        assert!(!Hct::is_blue(100.0));
        assert!(Hct::is_yellow(110.0));
        assert!(Hct::is_cyan(180.0));
    }

    #[test]
    fn test_hct_roundtrip_in_gamut() {
        let hue = 67.0;
        let chroma = 20.0;
        let tone = 52.0;
        let hct = Hct::new(hue, chroma, tone);

        // HCT -> RGB -> HCT should be stable for in-gamut colors
        let argb = hct.to_argb();
        let argb_string = format!("{:X}", argb.0);
        assert_eq!(argb_string, "FF967655");
        let back_convert = Hct::from_argb(argb);

        assert!((back_convert.hue - hue).abs() < 0.5);
        assert!((back_convert.chroma - chroma).abs() < 0.5);
        assert!((back_convert.tone - tone).abs() < 0.5);
    }

    #[test]
    fn test_hct_clipping() {
        // HCT(67, 91, 52) is out of gamut in sRGB.
        // It should be clipped to the maximum possible chroma (~49.2).
        let hct = Hct::new(67.0, 91.0, 52.0);

        assert!((hct.hue() - 67.0).abs() < 1.0);
        assert!(hct.chroma() < 50.0); // Clipped!
        assert!((hct.tone() - 52.0).abs() < 1.0);

        // The resulting ARGB should be #B26C00
        assert_eq!(format!("{:X}", hct.to_argb().0), "FFB26C00");
    }
}
