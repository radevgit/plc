//! Configuration for the rule detector.
//!
//! Configuration can be loaded from a `plceye.toml` file.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{Error, Result};
use crate::error::ConfigErrorKind;

/// Main configuration for the rule detector.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleConfig {
    /// Global settings
    pub general: GeneralConfig,

    /// Unused tag detection settings
    pub unused_tags: UnusedTagsConfig,

    /// Undefined tag detection settings
    pub undefined_tags: UndefinedTagsConfig,

    /// Empty routine detection settings
    pub empty_routines: EmptyRoutinesConfig,
}

impl RuleConfig {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::FileRead {
            path: path.display().to_string(),
            source: e,
        })?;
        Self::parse(&content)
    }

    /// Parse configuration from a TOML string.
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|_| Error::ConfigParse {
            kind: ConfigErrorKind::TomlSyntax,
        })
    }

    /// Generate a default configuration file as a string.
    pub fn default_toml() -> String {
        r#"# plceye.toml - PLC Code Rule Detector Configuration

[general]
# Minimum severity to report: "info", "warning", "error"
min_severity = "info"

[unused_tags]
# Enable unused tag detection
enabled = true

# Ignore tags matching these patterns (glob-style)
ignore_patterns = [
    "_*",           # Tags starting with underscore (often internal)
    "HMI_*",        # HMI interface tags
]

# Ignore tags in these scopes
ignore_scopes = [
    # "Program:MainProgram",  # Example: ignore MainProgram
]

[undefined_tags]
# Enable undefined tag detection (tags referenced but not declared)
enabled = true

# Ignore undefined tags matching these patterns (useful for I/O)
ignore_patterns = [
    "Local:*",      # Module I/O references
]

[empty_routines]
# Enable empty routine detection
enabled = true

# Ignore routines matching these patterns
ignore_patterns = [
    # "Unused_*",    # Example: ignore placeholder routines
]
"#
        .to_string()
    }
}

/// General configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Minimum severity level to report.
    pub min_severity: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            min_severity: "info".to_string(),
        }
    }
}

/// Configuration for unused tag detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnusedTagsConfig {
    /// Whether this detector is enabled.
    pub enabled: bool,

    /// Glob patterns for tags to ignore.
    pub ignore_patterns: Vec<String>,

    /// Scopes to ignore (e.g., "Program:MainProgram").
    pub ignore_scopes: Vec<String>,
}

impl Default for UnusedTagsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore_patterns: vec!["_*".to_string()],
            ignore_scopes: vec![],
        }
    }
}

/// Configuration for undefined tag detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UndefinedTagsConfig {
    /// Whether this detector is enabled.
    pub enabled: bool,

    /// Glob patterns for tags to ignore.
    pub ignore_patterns: Vec<String>,
}

impl Default for UndefinedTagsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore_patterns: vec!["Local:*".to_string()],
        }
    }
}

/// Configuration for empty routine detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmptyRoutinesConfig {
    /// Whether this detector is enabled.
    pub enabled: bool,

    /// Glob patterns for routines to ignore.
    pub ignore_patterns: Vec<String>,
}

impl Default for EmptyRoutinesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ignore_patterns: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuleConfig::default();
        assert!(config.unused_tags.enabled);
        assert_eq!(config.general.min_severity, "info");
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[general]
min_severity = "warning"

[unused_tags]
enabled = true
ignore_patterns = ["Test_*", "Debug_*"]
ignore_scopes = ["Program:Debug"]
"#;
        let config = RuleConfig::parse(toml).unwrap();
        assert_eq!(config.general.min_severity, "warning");
        assert!(config.unused_tags.enabled);
        assert_eq!(config.unused_tags.ignore_patterns.len(), 2);
    }

    #[test]
    fn test_default_toml_parses() {
        let toml = RuleConfig::default_toml();
        let config = RuleConfig::parse(&toml).unwrap();
        assert!(config.unused_tags.enabled);
    }
}
