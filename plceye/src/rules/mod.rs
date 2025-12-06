//! Individual rule detectors.

mod complexity;
mod empty_routines;
mod nesting;
mod plcopen_rules;
mod undefined_tags;
mod unused_tags;

// L5X-specific detectors
pub use complexity::ComplexityDetector;
pub use empty_routines::EmptyRoutinesDetector;
pub use nesting::NestingDetector;
pub use undefined_tags::UndefinedTagsDetector;
pub use unused_tags::UnusedTagsDetector;

// PLCopen-specific detectors
pub use plcopen_rules::{
    PlcopenUnusedVarsDetector,
    PlcopenUndefinedVarsDetector,
    PlcopenEmptyPousDetector,
};
