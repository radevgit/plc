//! Conversion from PLCopen XML types to plcmodel.
//!
//! This module provides the [`ToPlcModel`] implementation for PLCopen projects,
//! converting parsed XML into the vendor-neutral `plcmodel` representation.

use plcmodel::{Pou, PouInterface, Project, ToPlcModel, Variable};
use iectypes::{PouType, VarClass};

use crate::{
    Project as PlcOpenProject,
    Root_project_InlineType_types_InlineType_pous_InlineType_pou_Inline as PlcOpenPou,
    VarListPlain_variable_Inline as PlcOpenVariable,
};

impl ToPlcModel for PlcOpenProject {
    fn to_plc_model(&self) -> Project {
        let name = self.file_header.as_ref()
            .map(|h| h.product_name.clone())
            .unwrap_or_else(|| "Unnamed".to_string());
        
        let mut project = Project::new(name);
        project.source_format = Some("PLCopen".to_string());
        
        // Convert POUs
        if let Some(ref types) = self.types {
            if let Some(ref pous) = types.pous {
                for pou in &pous.pou {
                    project.pous.push(convert_pou(pou));
                }
            }
        }
        
        project
    }
}

fn convert_pou(pou: &PlcOpenPou) -> Pou {
    let pou_type = match pou.pou_type.to_lowercase().as_str() {
        "program" => PouType::Program,
        "function" => PouType::Function,
        "functionblock" => PouType::FunctionBlock,
        _ => PouType::Program,
    };
    
    let mut result = Pou::new(&pou.name, pou_type);
    
    // Convert interface (variables)
    if let Some(ref interface) = pou.interface {
        result.interface = convert_interface(interface);
    }
    
    result
}

fn convert_interface(
    interface: &crate::Root_project_InlineType_types_InlineType_pous_InlineType_pou_InlineType_interface_Inline
) -> PouInterface {
    let mut pi = PouInterface::default();
    
    // Input variables
    for input_vars in &interface.input_vars {
        for var in &input_vars.variable {
            pi.inputs.push(convert_variable(var, VarClass::Input));
        }
    }
    
    // Output variables
    for output_vars in &interface.output_vars {
        for var in &output_vars.variable {
            pi.outputs.push(convert_variable(var, VarClass::Output));
        }
    }
    
    // InOut variables
    for inout_vars in &interface.in_out_vars {
        for var in &inout_vars.variable {
            pi.in_outs.push(convert_variable(var, VarClass::InOut));
        }
    }
    
    // Local variables
    for local_vars in &interface.local_vars {
        for var in &local_vars.variable {
            pi.locals.push(convert_variable(var, VarClass::Local));
        }
    }
    
    // Temp variables
    for temp_vars in &interface.temp_vars {
        for var in &temp_vars.variable {
            pi.temps.push(convert_variable(var, VarClass::Temp));
        }
    }
    
    // External variables
    for ext_vars in &interface.external_vars {
        for var in &ext_vars.variable {
            pi.externals.push(convert_variable(var, VarClass::External));
        }
    }
    
    pi
}

fn convert_variable(var: &PlcOpenVariable, var_class: VarClass) -> Variable {
    // Note: type info is not captured by codegen due to xs:choice/group issue
    // For now, use "ANY" as placeholder - real type extraction needs XML parsing
    let data_type = "ANY".to_string();
    
    let mut result = Variable::new(&var.name, data_type);
    result.var_class = var_class;
    
    // Address binding
    if let Some(ref addr) = var.address {
        result.address = Some(addr.clone());
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_str;

    #[test]
    fn test_convert_simple_project() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <project xmlns="http://www.plcopen.org/xml/tc6_0200">
            <fileHeader companyName="Test" productName="TestProject" productVersion="1.0" creationDateTime="2024-01-01T00:00:00"/>
            <contentHeader name="Test"/>
            <types>
                <pous>
                    <pou name="Main" pouType="program">
                        <interface>
                            <inputVars>
                                <variable name="Start">
                                    <type><BOOL/></type>
                                </variable>
                            </inputVars>
                            <localVars>
                                <variable name="Counter">
                                    <type><INT/></type>
                                </variable>
                            </localVars>
                        </interface>
                    </pou>
                </pous>
            </types>
        </project>"#;
        
        let plcopen: PlcOpenProject = from_str(xml).expect("Failed to parse");
        let model = plcopen.to_plc_model();
        
        assert_eq!(model.name, "TestProject");
        assert_eq!(model.pous.len(), 1);
        
        let main = &model.pous[0];
        assert_eq!(main.name, "Main");
        assert!(matches!(main.pou_type, PouType::Program));
        assert_eq!(main.interface.inputs.len(), 1);
        assert_eq!(main.interface.inputs[0].name, "Start");
        assert_eq!(main.interface.locals.len(), 1);
        assert_eq!(main.interface.locals[0].name, "Counter");
    }

    #[test]
    fn test_convert_real_file() {
        let path = std::path::Path::new(env!("HOME"))
            .join("devpublic/dataplc/plcopen/bbr/exemples/first_steps/plc.xml");
        
        if !path.exists() {
            eprintln!("Skipping test: file not found");
            return;
        }
        
        let xml = std::fs::read_to_string(&path).expect("Failed to read");
        let plcopen: PlcOpenProject = from_str(&xml).expect("Failed to parse");
        let model = plcopen.to_plc_model();
        
        assert!(!model.pous.is_empty());
        eprintln!("Converted {} POUs", model.pous.len());
        
        for pou in &model.pous {
            eprintln!("  {}: {:?}, {} inputs, {} outputs, {} locals",
                pou.name, pou.pou_type,
                pou.interface.inputs.len(),
                pou.interface.outputs.len(),
                pou.interface.locals.len());
        }
    }
}
