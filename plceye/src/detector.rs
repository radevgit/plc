//! Main rule detector that coordinates all individual detectors.

use std::path::Path;

use l5x::Controller;

use crate::analysis::{analyze_controller, analyze_plcopen_project, ParseStats, PlcopenStats};
use crate::config::RuleConfig;
use crate::loader::LoadedProject;
use crate::report::{Report, Severity};
use crate::rules::{
    ComplexityDetector, EmptyRoutinesDetector, UndefinedTagsDetector, UnusedTagsDetector,
    PlcopenUnusedVarsDetector, PlcopenUndefinedVarsDetector, PlcopenEmptyPousDetector,
};
use crate::Result;

/// Main rule detector that runs all enabled detectors.
pub struct RuleDetector {
    config: RuleConfig,
}

impl RuleDetector {
    /// Create a new rule detector with default configuration.
    pub fn new() -> Self {
        Self {
            config: RuleConfig::default(),
        }
    }

    /// Create a new rule detector with the given configuration.
    pub fn with_config(config: RuleConfig) -> Self {
        Self { config }
    }

    /// Load configuration from a file.
    pub fn from_config_file(path: &Path) -> Result<Self> {
        let config = RuleConfig::from_file(path)?;
        Ok(Self { config })
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RuleConfig {
        &self.config
    }

    /// Get minimum severity from config.
    pub fn min_severity(&self) -> Severity {
        Severity::parse(&self.config.general.min_severity).unwrap_or(Severity::Info)
    }

    /// Analyze a file (L5X or PLCopen) and return a report.
    pub fn analyze_file(&self, path: &Path) -> Result<Report> {
        let project = LoadedProject::from_file(path)?;
        let mut report = self.analyze(&project)?;
        report.source_file = project.source_path;
        Ok(report)
    }

    /// Analyze a loaded project.
    pub fn analyze(&self, project: &LoadedProject) -> Result<Report> {
        if let Some(ref controller) = project.l5x_controller {
            return self.analyze_controller(controller);
        }
        
        if let Some(ref plcopen) = project.plcopen_project {
            return self.analyze_plcopen(plcopen, project.source_path.clone());
        }
        
        // Unknown format
        Ok(Report::new())
    }
    
    /// Analyze a PLCopen project.
    fn analyze_plcopen(&self, project: &plcopen::Project, source_path: Option<String>) -> Result<Report> {
        let analysis = analyze_plcopen_project(project);
        
        let mut report = Report::new();
        report.source_file = source_path;
        
        // Run PLCopen-specific detectors
        let unused_detector = PlcopenUnusedVarsDetector::new(&self.config.unused_tags);
        unused_detector.detect(&analysis, &mut report);
        
        let undefined_detector = PlcopenUndefinedVarsDetector::new(&self.config.undefined_tags);
        undefined_detector.detect(&analysis, &mut report);
        
        let empty_detector = PlcopenEmptyPousDetector::new(&self.config.empty_routines);
        empty_detector.detect(&analysis, &mut report);
        
        Ok(report)
    }

    /// Analyze a parsed L5X controller.
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

        // Run cyclomatic complexity detector on ST routines
        let complexity_detector = ComplexityDetector::new(&self.config.complexity);
        complexity_detector.detect(&analysis, &mut report);

        Ok(report)
    }

    /// Get statistics for a file without running rule detection.
    pub fn get_stats_file(&self, path: &Path) -> Result<ParseStats> {
        let project = LoadedProject::from_file(path)?;
        self.get_stats(&project)
    }

    /// Get statistics for a loaded project (L5X format).
    pub fn get_stats(&self, project: &LoadedProject) -> Result<ParseStats> {
        if let Some(ref controller) = project.l5x_controller {
            let analysis = analyze_controller(controller);
            return Ok(analysis.stats);
        }
        
        // For non-L5X, return empty stats
        Ok(ParseStats::default())
    }
    
    /// Get PLCopen statistics for a loaded project.
    pub fn get_plcopen_stats(&self, project: &LoadedProject) -> Result<PlcopenStats> {
        if let Some(ref plcopen) = project.plcopen_project {
            let analysis = analyze_plcopen_project(plcopen);
            return Ok(analysis.stats);
        }
        
        Ok(PlcopenStats::default())
    }
}

impl Default for RuleDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_default() {
        let detector = RuleDetector::new();
        assert!(detector.config().unused_tags.enabled);
    }

    #[test]
    fn test_detector_with_config() {
        let mut config = RuleConfig::default();
        config.unused_tags.enabled = false;
        let detector = RuleDetector::with_config(config);
        assert!(!detector.config().unused_tags.enabled);
    }

    #[test]
    fn test_analyze_l5x() {
        let xml = r#"<?xml version="1.0"?>
        <RSLogix5000Content SchemaRevision="1.0" SoftwareRevision="32.00">
            <Controller Name="TestController">
                <Tags>
                    <Tag Name="Unused" DataType="BOOL"/>
                </Tags>
                <Programs>
                    <Program Name="MainProgram">
                        <Routines>
                            <Routine Name="MainRoutine" Type="RLL">
                                <RLLContent>
                                    <Rung Number="0">
                                        <Text>NOP();</Text>
                                    </Rung>
                                </RLLContent>
                            </Routine>
                        </Routines>
                    </Program>
                </Programs>
            </Controller>
        </RSLogix5000Content>"#;
        
        let project = LoadedProject::from_str(xml, None).expect("Should parse");
        let detector = RuleDetector::new();
        let report = detector.analyze(&project).expect("Should analyze");
        
        // Should detect unused tag
        assert!(!report.rules.is_empty());
    }

    #[test]
    fn test_analyze_plcopen() {
        let xml = r#"<?xml version="1.0"?>
        <project xmlns="http://www.plcopen.org/xml/tc6_0200">
            <fileHeader companyName="Test" productName="TestProject" productVersion="1.0" creationDateTime="2024-01-01T00:00:00"/>
            <contentHeader name="Test"/>
            <types>
                <pous>
                    <pou name="Main" pouType="program"/>
                </pous>
            </types>
        </project>"#;
        
        let project = LoadedProject::from_str(xml, None).expect("Should parse");
        let detector = RuleDetector::new();
        let report = detector.analyze(&project).expect("Should analyze");
        
        // Should detect empty POU
        assert!(report.rules.iter().any(|s| s.identifier == "Main"));
    }
}
