//! Project statistics.

use crate::Project;
use iectypes::PouType;

/// Statistics about a project.
#[derive(Debug, Clone, Default)]
pub struct ProjectStats {
    /// Number of programs
    pub programs: usize,
    /// Number of function blocks
    pub function_blocks: usize,
    /// Number of functions
    pub functions: usize,
    /// Total number of POUs
    pub total_pous: usize,
    /// Number of user-defined data types
    pub data_types: usize,
    /// Number of global variables
    pub global_vars: usize,
    /// Total variables across all POUs
    pub total_vars: usize,
    /// Number of tasks
    pub tasks: usize,
}

impl ProjectStats {
    /// Calculate statistics from a project.
    pub fn from_project(project: &Project) -> Self {
        let mut stats = Self::default();
        
        for pou in &project.pous {
            stats.total_pous += 1;
            match pou.pou_type {
                PouType::Program => stats.programs += 1,
                PouType::FunctionBlock => stats.function_blocks += 1,
                PouType::Function => stats.functions += 1,
            }
            stats.total_vars += pou.interface.all_variables().count();
        }
        
        stats.data_types = project.data_types.len();
        
        if let Some(ref config) = project.configuration {
            for resource in &config.resources {
                stats.tasks += resource.tasks.len();
                stats.global_vars += resource.global_vars.len();
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pou, Variable};
    use iectypes::PouType;

    #[test]
    fn test_stats_counting() {
        let mut project = Project::new("Test");
        
        let mut prog = Pou::new("Main", PouType::Program);
        prog.interface.locals.push(Variable::new("Counter", "INT"));
        project.pous.push(prog);
        
        project.pous.push(Pou::new("FB_Motor", PouType::FunctionBlock));
        project.pous.push(Pou::new("FC_Add", PouType::Function));
        
        let stats = ProjectStats::from_project(&project);
        
        assert_eq!(stats.programs, 1);
        assert_eq!(stats.function_blocks, 1);
        assert_eq!(stats.functions, 1);
        assert_eq!(stats.total_pous, 3);
        assert_eq!(stats.total_vars, 1);
    }
}
