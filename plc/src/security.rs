//! Security limits for PLC parsers to prevent DoS attacks

/// Parser security limits to prevent denial-of-service attacks
#[derive(Debug, Clone)]
pub struct ParserLimits {
    /// Maximum input size in bytes
    pub max_input_size: usize,
    /// Maximum loop iterations (for any single loop)
    pub max_iterations: usize,
    /// Maximum recursion/nesting depth
    pub max_depth: usize,
    /// Maximum items in any collection (Vec, etc)
    pub max_collection_size: usize,
    /// Maximum total nodes/elements parsed
    pub max_nodes: usize,
    /// Maximum string/text length
    pub max_string_length: usize,
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self::balanced()
    }
}

impl ParserLimits {
    /// Balanced limits suitable for most use cases
    pub fn balanced() -> Self {
        ParserLimits {
            max_input_size: 100 * 1024 * 1024,      // 100 MB
            max_iterations: 1_000_000,              // 1M iterations
            max_depth: 256,                         // 256 levels deep
            max_collection_size: 100_000,           // 100K items
            max_nodes: 10_000_000,                  // 10M nodes
            max_string_length: 1024 * 1024,         // 1 MB strings
        }
    }

    /// Strict limits for untrusted/external input
    pub fn strict() -> Self {
        ParserLimits {
            max_input_size: 10 * 1024 * 1024,       // 10 MB
            max_iterations: 100_000,                // 100K iterations
            max_depth: 64,                          // 64 levels
            max_collection_size: 10_000,            // 10K items
            max_nodes: 1_000_000,                   // 1M nodes
            max_string_length: 64 * 1024,           // 64 KB strings
        }
    }

    /// Relaxed limits for trusted internal input
    pub fn relaxed() -> Self {
        ParserLimits {
            max_input_size: 1024 * 1024 * 1024,     // 1 GB
            max_iterations: 10_000_000,             // 10M iterations
            max_depth: 512,                         // 512 levels
            max_collection_size: 1_000_000,         // 1M items
            max_nodes: 100_000_000,                 // 100M nodes
            max_string_length: 10 * 1024 * 1024,    // 10 MB strings
        }
    }

    /// Check if input size is within limits
    pub fn check_input_size(&self, size: usize) -> Result<(), SecurityError> {
        if size > self.max_input_size {
            return Err(SecurityError::InputTooLarge { 
                size, 
                limit: self.max_input_size 
            });
        }
        Ok(())
    }

    /// Check if depth is within limits
    pub fn check_depth(&self, depth: usize) -> Result<(), SecurityError> {
        if depth > self.max_depth {
            return Err(SecurityError::DepthExceeded { 
                depth, 
                limit: self.max_depth 
            });
        }
        Ok(())
    }

    /// Check if collection size is within limits
    pub fn check_collection_size(&self, size: usize, name: &str) -> Result<(), SecurityError> {
        if size >= self.max_collection_size {
            return Err(SecurityError::CollectionTooLarge { 
                collection: name.to_string(),
                size, 
                limit: self.max_collection_size 
            });
        }
        Ok(())
    }

    /// Check if total nodes is within limits
    pub fn check_node_count(&self, count: usize) -> Result<(), SecurityError> {
        if count > self.max_nodes {
            return Err(SecurityError::TooManyNodes { 
                count, 
                limit: self.max_nodes 
            });
        }
        Ok(())
    }

    /// Check if string length is within limits
    pub fn check_string_length(&self, length: usize) -> Result<(), SecurityError> {
        if length > self.max_string_length {
            return Err(SecurityError::StringTooLong { 
                length, 
                limit: self.max_string_length 
            });
        }
        Ok(())
    }
}

/// Security-related parsing errors
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    /// Input exceeds maximum allowed size
    InputTooLarge { size: usize, limit: usize },
    /// Nesting depth exceeds maximum
    DepthExceeded { depth: usize, limit: usize },
    /// Collection size exceeds maximum
    CollectionTooLarge { collection: String, size: usize, limit: usize },
    /// Too many nodes/elements parsed
    TooManyNodes { count: usize, limit: usize },
    /// String/text exceeds maximum length
    StringTooLong { length: usize, limit: usize },
    /// Too many loop iterations
    IterationLimitExceeded { iterations: usize, limit: usize },
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::InputTooLarge { size, limit } => {
                write!(f, "Input size {} bytes exceeds limit of {} bytes", size, limit)
            }
            SecurityError::DepthExceeded { depth, limit } => {
                write!(f, "Nesting depth {} exceeds limit of {}", depth, limit)
            }
            SecurityError::CollectionTooLarge { collection, size, limit } => {
                write!(f, "Collection '{}' with {} items exceeds limit of {}", collection, size, limit)
            }
            SecurityError::TooManyNodes { count, limit } => {
                write!(f, "Total node count {} exceeds limit of {}", count, limit)
            }
            SecurityError::StringTooLong { length, limit } => {
                write!(f, "String length {} exceeds limit of {}", length, limit)
            }
            SecurityError::IterationLimitExceeded { iterations, limit } => {
                write!(f, "Iterations {} exceeds limit of {}", iterations, limit)
            }
        }
    }
}

impl std::error::Error for SecurityError {}

/// Parser state that tracks security limits
#[derive(Debug)]
pub struct ParserState {
    limits: ParserLimits,
    depth: usize,
    nodes: usize,
}

impl ParserState {
    /// Create new parser state with given limits
    pub fn new(limits: ParserLimits) -> Self {
        ParserState {
            limits,
            depth: 0,
            nodes: 0,
        }
    }

    /// Create parser state with default limits
    pub fn default() -> Self {
        Self::new(ParserLimits::default())
    }

    /// Get current limits
    pub fn limits(&self) -> &ParserLimits {
        &self.limits
    }

    /// Enter a new nesting level
    pub fn enter_depth(&mut self) -> Result<(), SecurityError> {
        self.depth += 1;
        self.limits.check_depth(self.depth)
    }

    /// Exit a nesting level
    pub fn exit_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Record a parsed node
    pub fn count_node(&mut self) -> Result<(), SecurityError> {
        self.nodes += 1;
        self.limits.check_node_count(self.nodes)
    }

    /// Check collection size
    pub fn check_collection(&self, size: usize, name: &str) -> Result<(), SecurityError> {
        self.limits.check_collection_size(size, name)
    }

    /// Check string length
    pub fn check_string(&self, length: usize) -> Result<(), SecurityError> {
        self.limits.check_string_length(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ParserLimits::default();
        assert!(limits.max_input_size > 0);
        assert!(limits.max_depth > 0);
    }

    #[test]
    fn test_strict_limits_smaller() {
        let strict = ParserLimits::strict();
        let relaxed = ParserLimits::relaxed();
        assert!(strict.max_input_size < relaxed.max_input_size);
        assert!(strict.max_depth < relaxed.max_depth);
    }

    #[test]
    fn test_input_size_check() {
        let limits = ParserLimits::strict();
        assert!(limits.check_input_size(100).is_ok());
        assert!(limits.check_input_size(limits.max_input_size + 1).is_err());
    }

    #[test]
    fn test_depth_tracking() {
        let mut state = ParserState::new(ParserLimits {
            max_depth: 3,
            ..ParserLimits::default()
        });

        assert!(state.enter_depth().is_ok());  // depth = 1
        assert!(state.enter_depth().is_ok());  // depth = 2
        assert!(state.enter_depth().is_ok());  // depth = 3
        assert!(state.enter_depth().is_err()); // depth = 4, exceeds limit

        state.exit_depth();
        assert!(state.enter_depth().is_ok());  // depth = 3 again
    }
}
