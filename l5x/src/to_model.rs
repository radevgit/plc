//! Conversion from L5X types to plcmodel.
//!
//! This module provides the [`ToPlcModel`] implementation for L5X projects,
//! converting parsed Rockwell L5X files into the vendor-neutral `plcmodel` representation.

use plcmodel::{Pou, Project, ToPlcModel, Variable};
use iectypes::{PouType, VarClass};

use crate::{
    Project as L5xProject,
    AProgram,
    Tag,
    Routine as L5xRoutine,
    RoutineContent,
    RungCollection,
    RungContent,
    TextWideContent,
    UDIDefinitionContent,
    UDIDefinition,
};

impl ToPlcModel for L5xProject {
    fn to_plc_model(&self) -> Project {
        let name = self.controller.as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unnamed".to_string());
        
        let mut project = Project::new(name);
        project.source_format = Some("L5X".to_string());
        
        if let Some(ref controller) = self.controller {
            // Convert programs to POUs
            if let Some(ref programs) = controller.programs {
                for program in &programs.program {
                    project.pous.push(convert_program(program));
                }
            }
            
            // Convert AOIs to POUs (FunctionBlocks)
            if let Some(ref aois) = controller.add_on_instruction_definitions {
                for aoi in &aois.add_on_instruction_definition {
                    project.pous.push(convert_aoi(aoi));
                }
            }
        }
        
        project
    }
}

fn convert_program(program: &AProgram) -> Pou {
    let mut pou = Pou::new(&program.name, PouType::Program);
    
    // Program-scoped tags become locals
    if let Some(ref tags) = program.tags {
        for tag in &tags.tag {
            pou.interface.locals.push(convert_tag(tag, VarClass::Local));
        }
    }
    
    // Extract routine bodies and combine into a single body
    if let Some(ref routines) = program.routines {
        let mut body_parts = Vec::new();
        for routine in &routines.routine {
            if let Some(rll_text) = extract_routine_rll(routine) {
                body_parts.push(format!("// Routine: {}\n{}", routine.name, rll_text));
            }
        }
        if !body_parts.is_empty() {
            pou.body = Some(plcmodel::Body::Raw {
                language: "RLL".to_string(),
                content: body_parts.join("\n\n"),
            });
        }
    }
    
    pou
}

fn convert_aoi(aoi: &UDIDefinition) -> Pou {
    let mut pou = Pou::new(&aoi.name, PouType::FunctionBlock);
    
    // Process UDI content (parameters, local tags, routines)
    for content in &aoi.content {
        match content {
            UDIDefinitionContent::Parameters(params) => {
                for param in &params.parameter {
                    let var_class = match param.usage.as_str() {
                        "Input" => VarClass::Input,
                        "Output" => VarClass::Output,
                        "InOut" => VarClass::InOut,
                        _ => VarClass::Local,
                    };
                    
                    let data_type = param.data_type.clone().unwrap_or_else(|| "DINT".to_string());
                    let mut var = Variable::new(&param.name, data_type);
                    var.var_class = var_class.clone();
                    
                    match var_class {
                        VarClass::Input => pou.interface.inputs.push(var),
                        VarClass::Output => pou.interface.outputs.push(var),
                        VarClass::InOut => pou.interface.in_outs.push(var),
                        _ => pou.interface.locals.push(var),
                    }
                }
            }
            UDIDefinitionContent::LocalTags(local_tags) => {
                for tag in &local_tags.local_tag {
                    let data_type = tag.data_type.clone();
                    let mut var = Variable::new(&tag.name, data_type);
                    var.var_class = VarClass::Local;
                    pou.interface.locals.push(var);
                }
            }
            UDIDefinitionContent::Routines(routines) => {
                let mut body_parts = Vec::new();
                for routine in &routines.routine {
                    if let Some(rll_text) = extract_routine_rll(routine) {
                        body_parts.push(format!("// Routine: {}\n{}", routine.name, rll_text));
                    }
                }
                if !body_parts.is_empty() {
                    pou.body = Some(plcmodel::Body::Raw {
                        language: "RLL".to_string(),
                        content: body_parts.join("\n\n"),
                    });
                }
            }
            _ => {}
        }
    }
    
    pou
}

fn extract_routine_rll(routine: &L5xRoutine) -> Option<String> {
    for content in &routine.content {
        if let RoutineContent::RLLContent(rll) = content {
            return Some(extract_rll_text(rll));
        }
    }
    None
}

fn extract_rll_text(rll: &RungCollection) -> String {
    let mut rungs = Vec::new();
    
    for rung in &rll.rung {
        for content in &rung.content {
            if let RungContent::Text(text_wide) = content {
                // Extract text from TextWide content
                for tc in &text_wide.content {
                    if let TextWideContent::TextContent(text) = tc {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            rungs.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    
    rungs.join("\n")
}

fn convert_tag(tag: &Tag, default_class: VarClass) -> Variable {
    let data_type = tag.data_type.clone().unwrap_or_else(|| "DINT".to_string());
    let mut var = Variable::new(&tag.name, data_type);
    
    // Determine var class from Usage attribute
    var.var_class = match tag.usage.as_deref() {
        Some("Input") => VarClass::Input,
        Some("Output") => VarClass::Output,
        Some("InOut") => VarClass::InOut,
        _ => default_class,
    };
    
    // Handle array dimensions - parse to Vec<u32>
    if let Some(ref dims) = tag.dimensions {
        let parsed: Vec<u32> = dims
            .split(&[' ', ',', 'x'][..])
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if !parsed.is_empty() {
            var.dimensions = parsed;
        }
    }
    
    var
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_str;

    #[test]
    fn test_convert_simple_project() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <RSLogix5000Content SchemaRevision="1.0" SoftwareRevision="32.00">
            <Controller Name="MainController">
                <Programs>
                    <Program Name="MainProgram" MainRoutineName="MainRoutine">
                        <Tags>
                            <Tag Name="Counter" DataType="DINT"/>
                            <Tag Name="Running" DataType="BOOL"/>
                        </Tags>
                        <Routines>
                            <Routine Name="MainRoutine" Type="RLL">
                                <RLLContent>
                                    <Rung Number="0">
                                        <Text>XIC(Start)OTE(Motor);</Text>
                                    </Rung>
                                </RLLContent>
                            </Routine>
                        </Routines>
                    </Program>
                </Programs>
            </Controller>
        </RSLogix5000Content>"#;
        
        let l5x: L5xProject = from_str(xml).expect("Failed to parse");
        let model = l5x.to_plc_model();
        
        assert_eq!(model.name, "MainController");
        assert_eq!(model.pous.len(), 1);
        
        let main = &model.pous[0];
        assert_eq!(main.name, "MainProgram");
        assert!(matches!(main.pou_type, PouType::Program));
        assert_eq!(main.interface.locals.len(), 2);
        assert!(main.body.is_some());
    }

    #[test]
    fn test_convert_real_file() {
        let path = std::path::Path::new(env!("HOME"))
            .join("devpublic/dataplc/l5x/sample-project.L5X");
        
        if !path.exists() {
            eprintln!("Skipping test: file not found at {:?}", path);
            return;
        }
        
        let xml = std::fs::read_to_string(&path).expect("Failed to read");
        let l5x: L5xProject = from_str(&xml).expect("Failed to parse");
        let model = l5x.to_plc_model();
        
        assert!(!model.name.is_empty());
        eprintln!("Converted project: {}", model.name);
        eprintln!("  {} POUs", model.pous.len());
        
        for pou in &model.pous {
            eprintln!("  {}: {:?}, {} locals, has_body={}",
                pou.name, pou.pou_type,
                pou.interface.locals.len(),
                pou.body.is_some());
        }
    }
}
