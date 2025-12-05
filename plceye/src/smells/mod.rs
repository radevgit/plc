//! Individual smell detectors.

mod empty_routines;
mod plcopen_smells;
mod undefined_tags;
mod unused_tags;

// L5X-specific detectors
pub use empty_routines::EmptyRoutinesDetector;
pub use undefined_tags::UndefinedTagsDetector;
pub use unused_tags::UnusedTagsDetector;

// PLCopen-specific detectors
pub use plcopen_smells::{
    PlcopenUnusedVarsDetector,
    PlcopenUndefinedVarsDetector,
    PlcopenEmptyPousDetector,
};
