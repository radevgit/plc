//! Task configuration.

/// A task definition for scheduling POUs.
///
/// Maps to:
/// - L5X: `<Task>` elements
/// - PLCopen: `<task>` within resources
#[derive(Debug, Clone)]
pub struct Task {
    /// Task name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Task priority (lower = higher priority, typically 1-15)
    pub priority: u8,

    /// What triggers the task
    pub trigger: TaskTrigger,

    /// Watchdog time in milliseconds
    pub watchdog_ms: Option<u32>,

    /// Programs assigned to this task
    pub programs: Vec<String>,
}

impl Task {
    /// Create a periodic task.
    pub fn periodic(name: impl Into<String>, period_ms: u32) -> Self {
        Self {
            name: name.into(),
            description: None,
            priority: 10,
            trigger: TaskTrigger::Periodic { period_ms },
            watchdog_ms: None,
            programs: Vec::new(),
        }
    }

    /// Create a continuous task.
    pub fn continuous(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            priority: 15, // Lowest priority
            trigger: TaskTrigger::Continuous,
            watchdog_ms: None,
            programs: Vec::new(),
        }
    }

    /// Create an event-triggered task.
    pub fn event(name: impl Into<String>, trigger_tag: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            priority: 5,
            trigger: TaskTrigger::Event {
                trigger_tag: trigger_tag.into(),
            },
            watchdog_ms: None,
            programs: Vec::new(),
        }
    }

    /// Add a program to this task.
    pub fn with_program(mut self, program: impl Into<String>) -> Self {
        self.programs.push(program.into());
        self
    }
}

/// What triggers task execution.
#[derive(Debug, Clone)]
pub enum TaskTrigger {
    /// Runs at a fixed time interval
    Periodic {
        /// Period in milliseconds
        period_ms: u32,
    },

    /// Runs continuously (free-running)
    Continuous,

    /// Triggered by an event/tag
    Event {
        /// Tag that triggers this task
        trigger_tag: String,
    },

    /// Triggered by motion group updates
    MotionGroup {
        /// Motion group name
        group_name: String,
    },
}

impl TaskTrigger {
    /// Get the period in ms, if this is a periodic task.
    pub fn period_ms(&self) -> Option<u32> {
        match self {
            TaskTrigger::Periodic { period_ms } => Some(*period_ms),
            _ => None,
        }
    }
}
