//! Custom SVG generation module
//!
//! Provides full control over SVG output with clean, valid SVG.

mod builder;
mod elements;
mod style;

pub use builder::SvgBuilder;
pub use elements::*;
pub use style::{Style, Color};
