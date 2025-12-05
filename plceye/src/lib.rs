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
//! ## CLI Usage
//!
//! ```bash
//! # Analyze a file
//! plceye project.L5X
//!
//! # With custom config
//! plceye --config plceye.toml project.L5X
//!
//! # JSON output
//! plceye --format json project.L5X
//! ```

pub mod analysis;
mod config;
mod detector;
mod error;
mod loader;
mod report;
mod smells;

pub use analysis::{ProjectAnalysis, analyze_controller};
pub use config::SmellConfig;
pub use detector::SmellDetector;
pub use error::{Error, Result, L5xParseErrorKind, ConfigErrorKind};
pub use loader::{LoadedProject, FileFormat};
pub use report::{Report, Smell, Severity, SmellKind};
pub use analysis::ParseStats;
