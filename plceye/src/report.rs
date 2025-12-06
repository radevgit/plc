//! Report types for rule detection results.

use std::fmt;

/// Severity level of a detected rule.
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

/// Kind of code rule detected.
///
/// This enum contains all rule types. The open-source `plceye` detects the first 5 rules.
/// Additional rules are detected by `plceye-pro` (commercial license).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleKind {
    // =========================================================================
    // OPEN SOURCE RULES (detected by plceye)
    // =========================================================================

    /// S0001: Tag is defined but never used
    UnusedTag,
    /// S0002: Tag is used but never defined (might be external/aliased)
    UndefinedTag,
    /// S0003: Empty routine or POU
    EmptyBlock,
    /// S0004: AOI is defined but never called
    UnusedAoi,
    /// S0005: DataType is defined but never used
    UnusedDataType,

    // =========================================================================
    // PRO RULES (detected by plceye-pro - commercial license)
    // For licensing information, contact: []
    // =========================================================================

    // --- Coding Practice (C) ---
    /// C0010: Floating-point comparison with = or <>
    FloatEquality,
    /// C0011: TIME comparison with = or <>
    TimeEquality,
    /// C0014: Possible division by zero
    DivisionByZero,
    /// C0015: Magic number (hardcoded literal that should be a constant)
    MagicNumber,
    /// C0016: Timer/counter without reset path
    TimerNoReset,
    /// C0031: POU calls itself recursively
    RecursiveCall,
    /// C0032: FOR loop variable modified inside loop
    LoopVarModified,
    /// C0050: POU has too many parameters (>7)
    TooManyParameters,
    /// C0060: Too many global variables
    ExcessiveGlobals,

    // --- Style (S) ---
    /// S0020: CONTINUE statement used
    ContinueUsed,
    /// S0021: EXIT statement used
    ExitUsed,
    /// S0022: IF without ELSE clause
    IfWithoutElse,
    /// S0023: CASE without ELSE clause
    CaseWithoutElse,
    /// S0025: RETURN not at end of POU
    ReturnInMiddle,

    // --- Metrics (M) ---
    /// M0001: Cyclomatic complexity too high
    CyclomaticComplexity,
    /// M0003: Deep nesting (> 4 levels)
    DeepNesting,

    // --- Naming (N) ---
    /// N0006: Name length < 3 characters
    NameTooShort,
    /// N0007: Name length > 30 characters
    NameTooLong,

    // --- Vendor-Specific L5X (X) ---
    /// X0001: AOI without description
    AoiNoDescription,
    /// X0002: Tag without description
    TagNoDescription,
    /// X0003: Routine without description
    RoutineNoDescription,
    /// X0004: Program without description
    ProgramNoDescription,
    /// X0006: Task watchdog disabled
    TaskWatchdogDisabled,
    /// X0007: Excessive task rate (<1ms)
    ExcessiveTaskRate,
    /// X0009: Alias chain (alias pointing to alias)
    AliasChain,
    /// X0010: Large array (>10000 elements)
    LargeArray,
}

impl RuleKind {
    /// Get the rule code (e.g., "S0001").
    pub fn code(&self) -> &'static str {
        match self {
            // Open Source rules
            RuleKind::UnusedTag => "S0001",
            RuleKind::UndefinedTag => "S0002",
            RuleKind::EmptyBlock => "S0003",
            RuleKind::UnusedAoi => "S0004",
            RuleKind::UnusedDataType => "S0005",
            // Pro: Coding Practice
            RuleKind::FloatEquality => "C0010",
            RuleKind::TimeEquality => "C0011",
            RuleKind::DivisionByZero => "C0014",
            RuleKind::MagicNumber => "C0015",
            RuleKind::TimerNoReset => "C0016",
            RuleKind::RecursiveCall => "C0031",
            RuleKind::LoopVarModified => "C0032",
            RuleKind::TooManyParameters => "C0050",
            RuleKind::ExcessiveGlobals => "C0060",
            // Pro: Style
            RuleKind::ContinueUsed => "S0020",
            RuleKind::ExitUsed => "S0021",
            RuleKind::IfWithoutElse => "S0022",
            RuleKind::CaseWithoutElse => "S0023",
            RuleKind::ReturnInMiddle => "S0025",
            // Pro: Metrics
            RuleKind::CyclomaticComplexity => "M0001",
            RuleKind::DeepNesting => "M0003",
            // Pro: Naming
            RuleKind::NameTooShort => "N0006",
            RuleKind::NameTooLong => "N0007",
            // Pro: Vendor-Specific L5X
            RuleKind::AoiNoDescription => "X0001",
            RuleKind::TagNoDescription => "X0002",
            RuleKind::RoutineNoDescription => "X0003",
            RuleKind::ProgramNoDescription => "X0004",
            RuleKind::TaskWatchdogDisabled => "X0006",
            RuleKind::ExcessiveTaskRate => "X0007",
            RuleKind::AliasChain => "X0009",
            RuleKind::LargeArray => "X0010",
        }
    }
    
    /// Get the rule name (e.g., "unused-tag").
    pub fn name(&self) -> &'static str {
        match self {
            // Coding Practice
            RuleKind::UnusedTag => "unused-tag",
            RuleKind::UndefinedTag => "undefined-tag",
            RuleKind::EmptyBlock => "empty-block",
            RuleKind::UnusedAoi => "unused-aoi",
            RuleKind::UnusedDataType => "unused-datatype",
            RuleKind::FloatEquality => "float-equality",
            RuleKind::TimeEquality => "time-equality",
            RuleKind::DivisionByZero => "division-by-zero",
            RuleKind::MagicNumber => "magic-number",
            RuleKind::TimerNoReset => "timer-no-reset",
            RuleKind::RecursiveCall => "recursive-call",
            RuleKind::LoopVarModified => "loop-var-modified",
            RuleKind::TooManyParameters => "too-many-parameters",
            RuleKind::ExcessiveGlobals => "excessive-globals",
            // Style
            RuleKind::ContinueUsed => "continue-used",
            RuleKind::ExitUsed => "exit-used",
            RuleKind::IfWithoutElse => "if-without-else",
            RuleKind::CaseWithoutElse => "case-without-else",
            RuleKind::ReturnInMiddle => "return-in-middle",
            // Metrics
            RuleKind::CyclomaticComplexity => "cyclomatic-complexity",
            RuleKind::DeepNesting => "deep-nesting",
            // Naming
            RuleKind::NameTooShort => "name-too-short",
            RuleKind::NameTooLong => "name-too-long",
            // Vendor-Specific L5X
            RuleKind::AoiNoDescription => "aoi-no-description",
            RuleKind::TagNoDescription => "tag-no-description",
            RuleKind::RoutineNoDescription => "routine-no-description",
            RuleKind::ProgramNoDescription => "program-no-description",
            RuleKind::TaskWatchdogDisabled => "task-watchdog-disabled",
            RuleKind::ExcessiveTaskRate => "excessive-task-rate",
            RuleKind::AliasChain => "alias-chain",
            RuleKind::LargeArray => "large-array",
        }
    }
}

impl fmt::Display for RuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// A single detected code rule.
#[derive(Debug, Clone)]
pub struct Rule {
    /// Kind of rule
    pub kind: RuleKind,
    /// Severity level
    pub severity: Severity,
    /// Location in the project (e.g., "Program:Main")
    pub location: String,
    /// The identifier involved (tag name, routine name, etc.)
    pub identifier: String,
    /// Human-readable message
    pub message: String,
}

impl Rule {
    /// Create a new rule.
    pub fn new(
        kind: RuleKind,
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

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} - {} ({})",
            self.severity, self.kind, self.location, self.message, self.identifier
        )
    }
}

/// Report containing all detected rules.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// All detected rules
    pub rules: Vec<Rule>,
    /// Source file that was analyzed
    pub source_file: Option<String>,
}

impl Report {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule to the report.
    pub fn add(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Filter rules by minimum severity.
    pub fn filter_by_severity(&self, min_severity: Severity) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|s| s.severity >= min_severity)
            .collect()
    }

    /// Check if report has any rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get total number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }
}
