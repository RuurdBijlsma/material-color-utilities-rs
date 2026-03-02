/// Themes for Dynamic Color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// grayscale
    Monochrome,
    /// near-neutral palette
    Neutral,
    /// calm, sedated colors
    TonalSpot,
    ///  highly saturated
    Vibrant,
    /// highly colorful, playful
    Expressive,
    /// maximally faithful to source color
    Fidelity,
    /// colors derived closely from the source color
    Content,
    /// rainbow-like palette
    Rainbow,
    /// multiple hues
    FruitSalad,
    /// not sure what this one does
    Cmf,
}