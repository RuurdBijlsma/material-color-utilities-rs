#![deny(clippy::unwrap_used)]
#![allow(
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::many_single_char_names,
    clippy::while_float,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::match_wildcard_for_single_variants,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_long_first_doc_paragraph,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
pub mod blend;
pub mod contrast;
pub mod dislike;
pub mod dynamic;
pub mod hct;
pub mod palettes;
pub mod quantize;
pub mod scheme;
pub mod score;
pub mod temperature;
pub mod utils;
pub mod theme;
