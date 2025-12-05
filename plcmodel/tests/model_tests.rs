//! Tests for plcmodel crate.

use iectypes::PouType;
use plcmodel::*;

#[test]
fn test_project_creation() {
    let mut project = Project::new("TestProject");
    project.description = Some("A test project".into());

    assert_eq!(project.name, "TestProject");
    assert_eq!(project.description.as_deref(), Some("A test project"));
    assert!(project.pous.is_empty());
}

#[test]
fn test_pou_creation() {
    let mut pou = Pou::new("MainProgram", PouType::Program);
    pou.description = Some("Main control program".into());

    assert_eq!(pou.name, "MainProgram");
    assert!(matches!(pou.pou_type, PouType::Program));
    assert!(pou.is_empty());
}

#[test]
fn test_pou_interface() {
    let mut pou = Pou::new("Motor_FB", PouType::FunctionBlock);

    pou.interface.inputs.push(Variable::input("Enable", "BOOL"));
    pou.interface.inputs.push(Variable::input("Speed", "REAL"));
    pou.interface.outputs.push(Variable::output("Running", "BOOL"));
    pou.interface.locals.push(Variable::new("Timer", "TON"));

    assert_eq!(pou.interface.variable_count(), 4);
    assert!(pou.find_variable("Enable").is_some());
    assert!(pou.find_variable("Running").is_some());
    assert!(pou.find_variable("NonExistent").is_none());
}

#[test]
fn test_variable_array() {
    let mut var = Variable::new("Data", "INT");
    var.dimensions = vec![10, 20];

    assert!(var.is_array());
    assert_eq!(var.array_size(), 200);
}

#[test]
fn test_data_type_struct() {
    let dt = DataTypeDef::structure(
        "MotorData",
        vec![
            StructMember::new("Speed", "REAL"),
            StructMember::new("Running", "BOOL"),
            StructMember::new("Faults", "DINT"),
        ],
    );

    assert_eq!(dt.name, "MotorData");
    if let DataTypeKind::Struct(s) = &dt.kind {
        assert_eq!(s.members.len(), 3);
    } else {
        panic!("Expected struct");
    }
}

#[test]
fn test_data_type_enum() {
    let dt = DataTypeDef::enumeration(
        "MachineState",
        vec![
            EnumMember::with_value("Stopped", 0),
            EnumMember::with_value("Running", 1),
            EnumMember::with_value("Faulted", 2),
        ],
    );

    assert_eq!(dt.name, "MachineState");
    if let DataTypeKind::Enum(e) = &dt.kind {
        assert_eq!(e.members.len(), 3);
        assert_eq!(e.members[1].value, Some(1));
    } else {
        panic!("Expected enum");
    }
}

#[test]
fn test_task_creation() {
    let task = Task::periodic("MainTask", 10)
        .with_program("MainProgram")
        .with_program("SafetyProgram");

    assert_eq!(task.name, "MainTask");
    assert_eq!(task.trigger.period_ms(), Some(10));
    assert_eq!(task.programs.len(), 2);
}

#[test]
fn test_body_empty() {
    assert!(Body::St("".into()).is_empty());
    assert!(Body::St("   \n  ".into()).is_empty());
    assert!(!Body::St("x := 1;".into()).is_empty());
    assert!(Body::Ld(vec![]).is_empty());
}

#[test]
fn test_project_pou_filtering() {
    let mut project = Project::new("Test");
    project.pous.push(Pou::new("MainProg", PouType::Program));
    project.pous.push(Pou::new("Util", PouType::Function));
    project
        .pous
        .push(Pou::new("Motor_FB", PouType::FunctionBlock));
    project.pous.push(Pou::new("Safety", PouType::Program));

    assert_eq!(project.programs().count(), 2);
    assert_eq!(project.functions().count(), 1);
    assert_eq!(project.function_blocks().count(), 1);

    assert!(project.find_pou("Motor_FB").is_some());
    assert!(project.find_pou("NonExistent").is_none());
}
