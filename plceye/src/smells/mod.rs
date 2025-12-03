//! Individual smell detectors.

mod empty_routines;
mod undefined_tags;
mod unused_tags;

pub use empty_routines::EmptyRoutinesDetector;
pub use undefined_tags::UndefinedTagsDetector;
pub use unused_tags::UnusedTagsDetector;
