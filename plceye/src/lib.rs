//! plceye - PLC Code Smell Detector
//!
//! Open source static analyzer for L5X files that detects code smells
//! and potential issues in PLC programs.
//!
//! ## Supported Smells
//!
//! - **unused_tags** - Tags defined but never used
//! - **undefined_tags** - Tags referenced but not defined
//! - **empty_routines** - Routines with no logic
//!
//! ## Usage
//!
//! ```ignore
//! use plceye::{SmellDetector, SmellConfig};
//!
//! let detector = SmellDetector::new();
//! let report = detector.analyze_file("project.L5X")?;
//!
//! for smell in &report.smells {
//!     println!("{}", smell);
//! }
//! ```

mod config;
mod detector;
mod error;
mod report;
mod smells;

pub use config::SmellConfig;
pub use detector::SmellDetector;
pub use error::{Error, Result, L5xParseErrorKind, ConfigErrorKind};
pub use report::{Report, Smell, Severity, SmellKind};
