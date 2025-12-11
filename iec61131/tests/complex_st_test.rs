use iec61131::{Parser, analysis::{CfgBuilder, max_nesting_depth}};

#[test]
fn test_complex_st_parsing() {
    let code = r#"
PROGRAM TestComplexity
VAR
    a : BOOL;
    b : BOOL;
    c : BOOL;
    d : BOOL;
    e : BOOL;
    result : INT;
END_VAR

IF a THEN
    IF b THEN
        IF c THEN
            IF d THEN
                IF e THEN
                    result := 1;
                ELSIF c THEN
                    result := 2;
                ELSE
                    result := 3;
                END_IF;
            END_IF;
        END_IF;
    END_IF;
END_IF;

FOR result := 1 TO 10 DO
    IF a THEN
        WHILE b DO
            CASE result OF
                1: c := TRUE;
                2: d := TRUE;
                3: e := TRUE;
            END_CASE;
        END_WHILE;
    END_IF;
END_FOR;

END_PROGRAM
"#;

    let mut parser = Parser::new(code);
    let cu = parser.parse().expect("Should parse successfully");
    
    assert_eq!(cu.declarations.len(), 1);
    
    if let iec61131::PouDeclaration::Program(prog) = &cu.declarations[0] {
        assert_eq!(prog.name, "TestComplexity");
        
        let cfg = CfgBuilder::new().build(&prog.body);
        let complexity = cfg.cyclomatic_complexity();
        let nesting = max_nesting_depth(&prog.body);
        
        println!("Cyclomatic complexity: {}", complexity);
        println!("Nesting depth: {}", nesting);
        
        // This code has deep nesting (5 levels) and high complexity
        assert!(nesting >= 5, "Expected nesting >= 5, got {}", nesting);
        assert!(complexity >= 10, "Expected complexity >= 10, got {}", complexity);
    } else {
        panic!("Expected Program declaration");
    }
}
