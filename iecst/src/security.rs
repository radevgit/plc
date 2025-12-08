//! Security limits for IEC 61131-3 ST parser to prevent DoS attacks

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
    /// Maximum total statements parsed
    pub max_statements: usize,
    /// Maximum string/identifier length
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
            max_statements: 10_000_000,             // 10M statements
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
            max_statements: 1_000_000,              // 1M statements
            max_string_length: 64 * 1024,           // 64 KB strings
        }
    }

    /// Relaxed limits for trusted/internal files
    pub fn relaxed() -> Self {
        ParserLimits {
            max_input_size: 500 * 1024 * 1024,      // 500 MB
            max_iterations: 10_000_000,             // 10M iterations
            max_depth: 512,                         // 512 levels
            max_collection_size: 1_000_000,         // 1M items
            max_statements: 100_000_000,            // 100M statements
            max_string_length: 10 * 1024 * 1024,    // 10 MB strings
        }
    }
}

/// Parser state tracker for enforcing security limits
#[derive(Debug)]
pub struct ParserState {
    limits: ParserLimits,
    current_depth: usize,
    statement_count: usize,
    iteration_count: usize,
}

impl ParserState {
    pub fn new(limits: ParserLimits) -> Self {
        ParserState {
            limits,
            current_depth: 0,
            statement_count: 0,
            iteration_count: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ParserLimits::default())
    }

    pub fn limits(&self) -> &ParserLimits {
        &self.limits
    }

    /// Enter a nested context (function, block, etc.)
    pub fn enter_depth(&mut self) -> Result<(), SecurityError> {
        self.current_depth += 1;
        if self.current_depth > self.limits.max_depth {
            return Err(SecurityError::DepthExceeded {
                current: self.current_depth,
                limit: self.limits.max_depth,
            });
        }
        Ok(())
    }

    /// Exit a nested context
    pub fn exit_depth(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Record a statement parsed
    pub fn record_statement(&mut self) -> Result<(), SecurityError> {
        self.statement_count += 1;
        if self.statement_count > self.limits.max_statements {
            return Err(SecurityError::TooManyStatements {
                count: self.statement_count,
                limit: self.limits.max_statements,
            });
        }
        Ok(())
    }

    /// Record a loop iteration
    pub fn record_iteration(&mut self) -> Result<(), SecurityError> {
        self.iteration_count += 1;
        if self.iteration_count > self.limits.max_iterations {
            return Err(SecurityError::TooManyIterations {
                count: self.iteration_count,
                limit: self.limits.max_iterations,
            });
        }
        Ok(())
    }

    /// Reset iteration counter (call at start of each loop)
    pub fn reset_iterations(&mut self) {
        self.iteration_count = 0;
    }

    /// Check collection size
    pub fn check_collection_size(&self, size: usize) -> Result<(), SecurityError> {
        if size > self.limits.max_collection_size {
            return Err(SecurityError::CollectionTooLarge {
                size,
                limit: self.limits.max_collection_size,
            });
        }
        Ok(())
    }

    /// Check string length
    pub fn check_string_length(&self, len: usize) -> Result<(), SecurityError> {
        if len > self.limits.max_string_length {
            return Err(SecurityError::StringTooLong {
                length: len,
                limit: self.limits.max_string_length,
            });
        }
        Ok(())
    }
}

/// Security-related errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Maximum nesting depth exceeded: {current} > {limit}")]
    DepthExceeded { current: usize, limit: usize },
    
    #[error("Too many statements: {count} > {limit}")]
    TooManyStatements { count: usize, limit: usize },
    
    #[error("Too many iterations: {count} > {limit}")]
    TooManyIterations { count: usize, limit: usize },
    
    #[error("Collection too large: {size} items > {limit}")]
    CollectionTooLarge { size: usize, limit: usize },
    
    #[error("String too long: {length} bytes > {limit}")]
    StringTooLong { length: usize, limit: usize },
    
    #[error("Input too large: {size} bytes > {limit}")]
    InputTooLarge { size: usize, limit: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ParserLimits::default();
        assert_eq!(limits.max_input_size, 100 * 1024 * 1024);
        assert_eq!(limits.max_depth, 256);
    }

    #[test]
    fn test_strict_limits() {
        let strict = ParserLimits::strict();
        let relaxed = ParserLimits::relaxed();
        assert!(strict.max_depth < relaxed.max_depth);
        assert!(strict.max_input_size < relaxed.max_input_size);
    }

    #[test]
    fn test_depth_tracking() {
        let limits = ParserLimits::strict();
        let mut state = ParserState::new(limits);
        
        for _ in 0..64 {
            assert!(state.enter_depth().is_ok());
        }
        assert!(state.enter_depth().is_err());
    }

    #[test]
    fn test_statement_counting() {
        let mut state = ParserState::new(ParserLimits {
            max_statements: 10,
            ..ParserLimits::default()
        });
        
        for _ in 0..10 {
            assert!(state.record_statement().is_ok());
        }
        assert!(state.record_statement().is_err());
    }
}
