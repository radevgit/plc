//! Conversion from PLCopen XML types to plcmodel.
//!
//! This module provides the [`ToPlcModel`] implementation for PLCopen projects,
//! converting parsed XML into the vendor-neutral `plcmodel` representation.

use plcmodel::{
    Body, DataTypeDef, DataTypeKind, Pou, PouInterface, Project, 
    StructDef, StructMember, ToPlcModel, Variable,
};
use iectypes::{PouType, VarClass};

use crate::{
    Project as PlcOpenProject,
    Root_project_InlineType_types_InlineType_pous_InlineType_pou_Inline as PlcOpenPou,
    VarListPlain_variable_Inline as PlcOpenVariable,
    VarList,
};

impl ToPlcModel for PlcOpenProject {
    fn to_plc_model(&self) -> Project {
        let mut project = Project::new(&self.file_header.as_ref()
            .and_then(|h| h.product_name.clone())
            .unwrap_or_else(|| "Unnamed".to_string()));
        
        project.source_format = Some("PLCopen".to_string());
        
        // Convert POUs
        if let Some(ref types) = self.types {
            if let Some(ref pous) = types.pous {
                for pou in &pous.pou {
                    project.pous.push(convert_pou(pou));
                }
            }
            
            // Convert data types
            if let Some(ref data_types) = types.data_types {
                for dt in &data_types.data_type {
                    if let Some(converted) = convert_data_type(dt) {
                        project.data_types.push(converted);
                    }
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
    
    // Convert body (we store ST code if present)
    if !pou.body.is_empty() {
        let body_xml = &pou.body[0];
        if body_xml.st.is_some() {
            // ST body - we could parse it here, but for now just note it exists
            result.body = Some(Body::default());
        }
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
    let data_type = extract_type_name(&var.r#type);
    
    let mut result = Variable::new(&var.name, data_type);
    result.var_class = var_class;
    
    // Address binding
    if let Some(ref addr) = var.address {
        result.address = Some(addr.clone());
    }
    
    result
}

/// Extract type name from PLCopen type element
fn extract_type_name(type_elem: &Option<crate::Data>) -> String {
    match type_elem {
        Some(data) => {
            // Check elementary types
            if data.bool.is_some() { return "BOOL".to_string(); }
            if data.sint.is_some() { return "SINT".to_string(); }
            if data.int.is_some() { return "INT".to_string(); }
            if data.dint.is_some() { return "DINT".to_string(); }
            if data.lint.is_some() { return "LINT".to_string(); }
            if data.usint.is_some() { return "USINT".to_string(); }
            if data.uint.is_some() { return "UINT".to_string(); }
            if data.udint.is_some() { return "UDINT".to_string(); }
            if data.ulint.is_some() { return "ULINT".to_string(); }
            if data.real.is_some() { return "REAL".to_string(); }
            if data.lreal.is_some() { return "LREAL".to_string(); }
            if data.time.is_some() { return "TIME".to_string(); }
            if data.date.is_some() { return "DATE".to_string(); }
            if data.dt.is_some() { return "DT".to_string(); }
            if data.tod.is_some() { return "TOD".to_string(); }
            if data.string.is_some() { return "STRING".to_string(); }
            if data.wstring.is_some() { return "WSTRING".to_string(); }
            if data.byte.is_some() { return "BYTE".to_string(); }
            if data.word.is_some() { return "WORD".to_string(); }
            if data.dword.is_some() { return "DWORD".to_string(); }
            if data.lword.is_some() { return "LWORD".to_string(); }
            
            // Derived type reference
            if let Some(ref derived) = data.derived {
                return derived.name.clone();
            }
            
            // Array type
            if let Some(ref arr) = data.array {
                if let Some(ref base) = arr.base_type {
                    let base_name = extract_type_name(&Some(base.clone()));
                    return format!("ARRAY OF {}", base_name);
                }
            }
            
            "ANY".to_string()
        }
        None => "ANY".to_string(),
    }
}

fn convert_data_type(
    dt: &crate::Root_project_InlineType_types_InlineType_dataTypes_InlineType_dataType_Inline
) -> Option<DataTypeDef> {
    let name = dt.name.clone();
    
    // Check if it's a structure
    if let Some(ref base) = dt.base_type {
        if let Some(ref struct_def) = base.r#struct {
            let members: Vec<StructMember> = struct_def.variable.iter().map(|v| {
                StructMember {
                    name: v.name.clone(),
                    data_type: extract_type_name(&v.r#type),
                    initial_value: None,
                    description: None,
                }
            }).collect();
            
            return Some(DataTypeDef {
                name,
                kind: DataTypeKind::Struct(StructDef { members }),
                description: None,
            });
        }
        
        // TODO: Handle enum, array, subrange types
    }
    
    None
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
        assert_eq!(main.interface.inputs[0].data_type, "BOOL");
        assert_eq!(main.interface.locals.len(), 1);
        assert_eq!(main.interface.locals[0].name, "Counter");
        assert_eq!(main.interface.locals[0].data_type, "INT");
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
