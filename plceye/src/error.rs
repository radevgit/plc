//! Error types for the rule detector.

use thiserror::Error;

/// Result type alias for rule operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the rule detector.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to read file
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse L5X file
    #[error("Failed to parse L5X file: {kind}")]
    L5xParse {
        kind: L5xParseErrorKind,
    },

    /// Failed to parse config file
    #[error("Failed to parse config file: {kind}")]
    ConfigParse {
        kind: ConfigErrorKind,
    },
}

/// Kinds of L5X parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum L5xParseErrorKind {
    /// XML deserialization failed
    XmlDeserialize,
    /// Missing required element
    MissingElement(&'static str),
}

impl std::fmt::Display for L5xParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L5xParseErrorKind::XmlDeserialize => write!(f, "XML deserialization failed"),
            L5xParseErrorKind::MissingElement(elem) => write!(f, "missing required element: {}", elem),
        }
    }
}

/// Kinds of configuration errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigErrorKind {
    /// TOML syntax error
    TomlSyntax,
}

impl std::fmt::Display for ConfigErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigErrorKind::TomlSyntax => write!(f, "TOML syntax error"),
        }
    }
}
