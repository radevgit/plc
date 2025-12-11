use iec61131::Parser;

#[test]
fn test_function_with_multiple_inputs() {
    let code = r#"
FUNCTION Multiply : REAL
    VAR_INPUT
        a : REAL;
        b : REAL;
        c : REAL;
    END_VAR
    
    Multiply := a * b * c;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let ast = result.unwrap();
    assert_eq!(ast.declarations.len(), 1);
}

#[test]
fn test_function_block_with_state() {
    let code = r#"
FUNCTION_BLOCK TON
    VAR_INPUT
        IN : BOOL;
        PT : TIME;
    END_VAR
    VAR_OUTPUT
        Q : BOOL;
        ET : TIME;
    END_VAR
    VAR
        startTime : TIME;
        running : BOOL;
    END_VAR
    
    IF IN AND NOT running THEN
        running := TRUE;
        startTime := 0;
    END_IF;
    
    IF running THEN
        ET := ET + 1;
        IF ET >= PT THEN
            Q := TRUE;
        END_IF;
    END_IF;
    
    IF NOT IN THEN
        running := FALSE;
        Q := FALSE;
        ET := 0;
    END_IF;
END_FUNCTION_BLOCK
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_program_with_case_statement() {
    let code = r#"
PROGRAM TrafficLight
    VAR
        state : INT := 0;
        redLight : BOOL;
        yellowLight : BOOL;
        greenLight : BOOL;
    END_VAR
    
    CASE state OF
        0:
            redLight := TRUE;
            yellowLight := FALSE;
            greenLight := FALSE;
        1:
            redLight := FALSE;
            yellowLight := TRUE;
            greenLight := FALSE;
        2:
            redLight := FALSE;
            yellowLight := FALSE;
            greenLight := TRUE;
    END_CASE;
    
    state := state + 1;
    IF state > 2 THEN
        state := 0;
    END_IF;
END_PROGRAM
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_for_loop() {
    let code = r#"
FUNCTION Sum : INT
    VAR_INPUT
        n : INT;
    END_VAR
    VAR
        i : INT;
        total : INT := 0;
    END_VAR
    
    FOR i := 1 TO n BY 1 DO
        total := total + i;
    END_FOR;
    
    Sum := total;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_while_loop() {
    let code = r#"
FUNCTION Factorial : INT
    VAR_INPUT
        n : INT;
    END_VAR
    VAR
        result : INT := 1;
        counter : INT;
    END_VAR
    
    counter := n;
    WHILE counter > 1 DO
        result := result * counter;
        counter := counter - 1;
    END_WHILE;
    
    Factorial := result;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_repeat_loop() {
    let code = r#"
FUNCTION FindValue : INT
    VAR_INPUT
        target : INT;
    END_VAR
    VAR
        value : INT := 0;
        found : BOOL := FALSE;
    END_VAR
    
    REPEAT
        value := value + 1;
        IF value = target THEN
            found := TRUE;
        END_IF;
    UNTIL found OR value > 100
    END_REPEAT;
    
    FindValue := value;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_array_type() {
    let code = r#"
FUNCTION_BLOCK DataBuffer
    VAR
        buffer : ARRAY[0..9] OF INT;
        index : INT := 0;
    END_VAR
    VAR_INPUT
        value : INT;
    END_VAR
    
    buffer[index] := value;
    index := index + 1;
    IF index > 9 THEN
        index := 0;
    END_IF;
END_FUNCTION_BLOCK
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_multiple_declarations() {
    let code = r#"
FUNCTION Add : INT
    VAR_INPUT
        a : INT;
        b : INT;
    END_VAR
    Add := a + b;
END_FUNCTION

FUNCTION Subtract : INT
    VAR_INPUT
        a : INT;
        b : INT;
    END_VAR
    Subtract := a - b;
END_FUNCTION

PROGRAM Main
    VAR
        x : INT := 10;
        y : INT := 5;
        result : INT;
    END_VAR
    
    result := x + y;
END_PROGRAM
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    
    let ast = result.unwrap();
    assert_eq!(ast.declarations.len(), 3);
}

#[test]
fn test_boolean_expressions() {
    let code = r#"
FUNCTION IsInRange : BOOL
    VAR_INPUT
        value : INT;
        min : INT;
        max : INT;
    END_VAR
    
    IsInRange := value >= min AND value <= max;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_complex_expression() {
    let code = r#"
FUNCTION Calculate : REAL
    VAR_INPUT
        a : REAL;
        b : REAL;
        c : REAL;
    END_VAR
    
    Calculate := (a + b) * c - (a - b) / c;
END_FUNCTION
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_string_type() {
    let code = r#"
FUNCTION_BLOCK MessageBuffer
    VAR
        message : STRING;
        length : INT;
    END_VAR
    VAR_INPUT
        newMessage : STRING;
    END_VAR
    
    message := newMessage;
    length := 0;
END_FUNCTION_BLOCK
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_time_type() {
    let code = r#"
FUNCTION_BLOCK Timer
    VAR
        elapsed : TIME;
        duration : TIME;
    END_VAR
    VAR_INPUT
        start : BOOL;
    END_VAR
    VAR_OUTPUT
        done : BOOL;
    END_VAR
    
    IF start THEN
        elapsed := 0;
        done := FALSE;
    END_IF;
END_FUNCTION_BLOCK
"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
