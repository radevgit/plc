//! Individual smell detectors.

mod empty_routines;
mod model_smells;
mod undefined_tags;
mod unused_tags;

// L5X-specific detectors
pub use empty_routines::EmptyRoutinesDetector;
pub use undefined_tags::UndefinedTagsDetector;
pub use unused_tags::UnusedTagsDetector;

// Model-based detectors (format-independent)
pub use model_smells::{
    ModelUnusedTagsDetector,
    ModelUndefinedTagsDetector,
    ModelEmptyRoutinesDetector,
};
