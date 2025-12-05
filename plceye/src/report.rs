//! Report types for smell detection results.

use std::fmt;

/// Severity level of a detected smell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational - potential issue worth reviewing
    Info,
    /// Warning - likely a problem
    Warning,
    /// Error - definite problem
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

impl Severity {
    /// Parse severity from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "info" => Some(Severity::Info),
            "warning" | "warn" => Some(Severity::Warning),
            "error" | "err" => Some(Severity::Error),
            _ => None,
        }
    }
}

/// Kind of code smell detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmellKind {
    /// C0001: Tag is defined but never used
    UnusedTag,
    /// C0002: Tag is used but never defined (might be external/aliased)
    UndefinedTag,
    /// C0003: Empty routine or POU
    EmptyBlock,
    /// C0004: AOI is defined but never called
    UnusedAoi,
    /// C0005: DataType is defined but never used
    UnusedDataType,
}

impl SmellKind {
    /// Get the rule code (e.g., "C0001").
    pub fn code(&self) -> &'static str {
        match self {
            SmellKind::UnusedTag => "C0001",
            SmellKind::UndefinedTag => "C0002",
            SmellKind::EmptyBlock => "C0003",
            SmellKind::UnusedAoi => "C0004",
            SmellKind::UnusedDataType => "C0005",
        }
    }
    
    /// Get the rule name (e.g., "unused-tag").
    pub fn name(&self) -> &'static str {
        match self {
            SmellKind::UnusedTag => "unused-tag",
            SmellKind::UndefinedTag => "undefined-tag",
            SmellKind::EmptyBlock => "empty-block",
            SmellKind::UnusedAoi => "unused-aoi",
            SmellKind::UnusedDataType => "unused-datatype",
        }
    }
}

impl fmt::Display for SmellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// A single detected code smell.
#[derive(Debug, Clone)]
pub struct Smell {
    /// Kind of smell
    pub kind: SmellKind,
    /// Severity level
    pub severity: Severity,
    /// Location in the project (e.g., "Program:Main")
    pub location: String,
    /// The identifier involved (tag name, routine name, etc.)
    pub identifier: String,
    /// Human-readable message
    pub message: String,
}

impl Smell {
    /// Create a new smell.
    pub fn new(
        kind: SmellKind,
        severity: Severity,
        location: impl Into<String>,
        identifier: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity,
            location: location.into(),
            identifier: identifier.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Smell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} - {} ({})",
            self.severity, self.kind, self.location, self.message, self.identifier
        )
    }
}

/// Report containing all detected smells.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// All detected smells
    pub smells: Vec<Smell>,
    /// Source file that was analyzed
    pub source_file: Option<String>,
}

impl Report {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a smell to the report.
    pub fn add(&mut self, smell: Smell) {
        self.smells.push(smell);
    }

    /// Filter smells by minimum severity.
    pub fn filter_by_severity(&self, min_severity: Severity) -> Vec<&Smell> {
        self.smells
            .iter()
            .filter(|s| s.severity >= min_severity)
            .collect()
    }

    /// Check if report has any smells.
    pub fn is_empty(&self) -> bool {
        self.smells.is_empty()
    }

    /// Get total number of smells.
    pub fn len(&self) -> usize {
        self.smells.len()
    }
}
