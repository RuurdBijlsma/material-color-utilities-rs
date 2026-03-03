use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeGenerationError {
    #[error("couldn't extract color from image")]
    CouldNotExtractColorFromImage,
}