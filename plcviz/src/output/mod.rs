//! SVG generation module
//!
//! Composable SVG builder with element functions.
//! No external dependencies - generates SVG as plain text.

mod elements;
mod builder;
mod style;

pub use elements::*;
pub use builder::SvgBuilder;
pub use style::{Style, Color};
