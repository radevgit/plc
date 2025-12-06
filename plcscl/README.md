# plcscl

Parser and AST for Siemens SCL (Structured Control Language).

SCL is Siemens' implementation of IEC 61131-3 Structured Text (ST) with proprietary extensions for S7-300/400/1200/1500 PLCs.

## Features

- Complete lexer for SCL tokens
- Recursive descent parser with operator precedence climbing
- Full AST representation
- Support for Siemens-specific extensions:
  - Pragmas: `{S7_Optimized_Access := 'TRUE'}`
  - Absolute addressing: `%I0.0`, `%Q0.1`, `%MW10`, `DB10.DBW0`
  - Regions: `REGION..END_REGION`
  - Data blocks (DB, FB, FC)
  - Multiple assignment operators: `:=`, `+=`, `-=`, `*=`, `/=`

## Language Support

### Block Types
- `FUNCTION` - Pure functions with return values
- `FUNCTION_BLOCK` - Stateful function blocks
- `DATA_BLOCK` - Data storage blocks
- `ORGANIZATION_BLOCK` - OB blocks (cyclic, startup, interrupt)
- `TYPE` - User-defined types

### Data Types
- **Elementary**: BOOL, BYTE, WORD, DWORD, LWORD, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, CHAR, WCHAR
- **Strings**: STRING[n], WSTRING[n]
- **Time**: TIME, LTIME, DATE, TIME_OF_DAY, DATE_AND_TIME
- **Complex**: ARRAY, STRUCT, POINTER, REF
- **User-defined types**

### Control Statements
- `IF..THEN..ELSIF..ELSE..END_IF`
- `CASE..OF..END_CASE` with ranges
- `FOR..TO..BY..DO..END_FOR`
- `WHILE..DO..END_WHILE`
- `REPEAT..UNTIL`
- `CONTINUE`, `EXIT`, `RETURN`
- `GOTO` with labels

### Operators
Precedence levels (11 = highest):
- Level 11: `**` (exponentiation)
- Level 10: `*`, `/`, `MOD`
- Level 9: `+`, `-`
- Level 8: `<`, `<=`, `>`, `>=`
- Level 7: `=`, `<>`
- Level 5: `AND`, `&`
- Level 4: `XOR`
- Level 3: `OR`

## Example

```rust
use plcscl::{Lexer, Parser};

let source = r#"
FUNCTION_BLOCK "PID_Controller"
{ S7_Optimized_Access := 'TRUE' }
VERSION : '1.0'

VAR_INPUT
    Setpoint : REAL;
    ProcessValue : REAL;
    Kp : REAL := 1.0;
    Ki : REAL := 0.1;
    Kd : REAL := 0.01;
END_VAR

VAR_OUTPUT
    Output : REAL;
END_VAR

VAR
    ErrorSum : REAL;
    LastError : REAL;
END_VAR

BEGIN
    // Calculate error
    #Error := #Setpoint - #ProcessValue;
    
    // PID calculation
    #Output := #Kp * #Error + 
               #Ki * #ErrorSum + 
               #Kd * (#Error - #LastError);
    
    // Update state
    #ErrorSum += #Error;
    #LastError := #Error;
    
    // Clamp output
    IF #Output > 100.0 THEN
        #Output := 100.0;
    ELSIF #Output < 0.0 THEN
        #Output := 0.0;
    END_IF;
END_FUNCTION_BLOCK
"#;

let lexer = Lexer::new(source);
let mut parser = Parser::new(lexer)?;
let ast = parser.parse()?;

println!("Parsed {} blocks", ast.blocks.len());
```


## License

MIT
