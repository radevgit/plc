//! Main smell detector that coordinates all individual detectors.

use std::path::Path;

use l5x::Controller;

use crate::analysis::{analyze_controller, ParseStats};
use crate::config::SmellConfig;
use crate::error::{Error, L5xParseErrorKind};
use crate::report::{Report, Severity};
use crate::smells::{EmptyRoutinesDetector, UndefinedTagsDetector, UnusedTagsDetector};
use crate::Result;

/// Main smell detector that runs all enabled detectors.
pub struct SmellDetector {
    config: SmellConfig,
}

impl SmellDetector {
    /// Create a new smell detector with default configuration.
    pub fn new() -> Self {
        Self {
            config: SmellConfig::default(),
        }
    }

    /// Create a new smell detector with the given configuration.
    pub fn with_config(config: SmellConfig) -> Self {
        Self { config }
    }

    /// Load configuration from a file.
    pub fn from_config_file(path: &Path) -> Result<Self> {
        let config = SmellConfig::from_file(path)?;
        Ok(Self { config })
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SmellConfig {
        &self.config
    }

    /// Get minimum severity from config.
    pub fn min_severity(&self) -> Severity {
        Severity::parse(&self.config.general.min_severity).unwrap_or(Severity::Info)
    }

    /// Analyze an L5X file and return a report.
    pub fn analyze_file(&self, path: &Path) -> Result<Report> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
            path: path.display().to_string(),
            source: e,
        })?;

        let mut report = self.analyze_str(&content)?;
        report.source_file = Some(path.display().to_string());
        Ok(report)
    }

    /// Analyze L5X content from a string.
    pub fn analyze_str(&self, content: &str) -> Result<Report> {
        // Parse the L5X file
        let project: l5x::Project = quick_xml::de::from_str(content)
            .map_err(|_| Error::L5xParse {
                kind: L5xParseErrorKind::XmlDeserialize,
            })?;

        let controller = project.controller.ok_or(Error::L5xParse {
            kind: L5xParseErrorKind::MissingElement("Controller"),
        })?;

        self.analyze_controller(&controller)
    }

    /// Analyze a parsed controller.
    pub fn analyze_controller(&self, controller: &Controller) -> Result<Report> {
        // Run the L5X analysis to get tag references, etc.
        let analysis = analyze_controller(controller);

        let mut report = Report::new();

        // Run unused tags detector
        let unused_tags_detector = UnusedTagsDetector::new(&self.config.unused_tags);
        unused_tags_detector.detect(controller, &analysis, &mut report);

        // Run undefined tags detector
        let undefined_tags_detector = UndefinedTagsDetector::new(&self.config.undefined_tags);
        undefined_tags_detector.detect(controller, &analysis, &mut report);

        // Run empty routines detector
        let empty_routines_detector = EmptyRoutinesDetector::new(&self.config.empty_routines);
        empty_routines_detector.detect(controller, &analysis, &mut report);

        Ok(report)
    }

    /// Get statistics for an L5X file without running smell detection.
    pub fn get_stats_file(&self, path: &Path) -> Result<ParseStats> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
            path: path.display().to_string(),
            source: e,
        })?;

        self.get_stats_str(&content)
    }

    /// Get statistics for L5X content from a string.
    pub fn get_stats_str(&self, content: &str) -> Result<ParseStats> {
        let project: l5x::Project = quick_xml::de::from_str(content)
            .map_err(|_| Error::L5xParse {
                kind: L5xParseErrorKind::XmlDeserialize,
            })?;

        let controller = project.controller.ok_or(Error::L5xParse {
            kind: L5xParseErrorKind::MissingElement("Controller"),
        })?;

        let analysis = analyze_controller(&controller);
        Ok(analysis.stats)
    }
}

impl Default for SmellDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_default() {
        let detector = SmellDetector::new();
        assert!(detector.config().unused_tags.enabled);
    }

    #[test]
    fn test_detector_with_config() {
        let mut config = SmellConfig::default();
        config.unused_tags.enabled = false;
        let detector = SmellDetector::with_config(config);
        assert!(!detector.config().unused_tags.enabled);
    }
}
