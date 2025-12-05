//! plceye - PLC Code Smell Detector
//!
//! Open source static analyzer for PLC files that detects code smells
//! and potential issues in PLC programs.
//!
//! ## Supported Formats
//!
//! - **L5X** - Rockwell Automation Studio 5000
//! - **PLCopen XML** - IEC 61131-3 standard exchange format
//!
//! ## Supported Smells
//!
//! - **unused_tags** - Tags defined but never used
//! - **undefined_tags** - Tags referenced but not defined
//! - **empty_routines** - Routines with no logic
//!
//! ## Library Usage
//!
//! ```rust,ignore
//! use plceye::{SmellDetector, LoadedProject};
//!
//! let project = LoadedProject::from_file("project.L5X")?;
//! let detector = SmellDetector::new();
//! let report = detector.analyze(&project)?;
//!
//! for smell in report.smells() {
//!     println!("{}", smell);
//! }
//! ```
//!
//! ## CLI Usage
//!
//! ```bash
//! # Analyze a file
//! plceye project.L5X
//!
//! # With custom config
//! plceye --config plceye.toml project.L5X
//! ```

pub mod analysis;
mod config;
mod detector;
mod error;
mod loader;
mod report;
mod smells;

// Core types
pub use config::SmellConfig;
pub use detector::SmellDetector;
pub use error::{Error, Result, L5xParseErrorKind, ConfigErrorKind};
pub use loader::{LoadedProject, FileFormat};
pub use report::{Report, Smell, Severity, SmellKind};

// Analysis types (for extensions)
pub use analysis::{ProjectAnalysis, ParseStats, analyze_controller};
pub use analysis::{PlcopenAnalysis, PlcopenStats, analyze_plcopen_project};

// Re-export parser crates for extensions
pub use l5x;
pub use plcopen;
