//! Security limits for SCL parser to prevent DoS attacks
//!
//! Provides protection against:
//! - Input size bombs (huge source files)
//! - Deep nesting attacks (excessive control structure nesting)
//! - Complexity attacks (too many statements/expressions)
//! - Memory exhaustion attacks

/// Security limits for parsing SCL code
#[derive(Debug, Clone)]
pub struct ParserLimits {
    /// Maximum input size in bytes
    pub max_input_size: usize,
    /// Maximum loop iterations (for any single loop during parsing)
    pub max_iterations: usize,
    /// Maximum recursion/nesting depth for control structures
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
    /// Balanced limits suitable for most industrial SCL programs
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

    /// Relaxed limits for trusted internal code
    pub fn relaxed() -> Self {
        ParserLimits {
            max_input_size: 1024 * 1024 * 1024,     // 1 GB
            max_iterations: 10_000_000,             // 10M iterations
            max_depth: 1024,                        // 1024 levels
            max_collection_size: 1_000_000,         // 1M items
            max_statements: 100_000_000,            // 100M statements
            max_string_length: 10 * 1024 * 1024,    // 10 MB strings
        }
    }
}

/// Runtime state for tracking parser security limits
#[derive(Debug)]
pub struct ParserState {
    limits: ParserLimits,
    current_depth: usize,
    statement_count: usize,
    iteration_count: usize,
}

impl ParserState {
    /// Create a new parser state with given limits
    pub fn new(limits: ParserLimits) -> Self {
        ParserState {
            limits,
            current_depth: 0,
            statement_count: 0,
            iteration_count: 0,
        }
    }

    /// Enter a new depth level (e.g., nested IF/FOR/WHILE block)
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

    /// Exit a depth level
    pub fn exit_depth(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Record a parsed statement
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

    /// Record an iteration (for parsing loops, not runtime loops)
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

    /// Check collection size before adding items
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
    pub fn check_string_length(&self, length: usize) -> Result<(), SecurityError> {
        if length > self.limits.max_string_length {
            return Err(SecurityError::StringTooLong {
                length,
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
