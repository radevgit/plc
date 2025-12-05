# iectypes

IEC 61131-3 data types, enums, and type definitions for PLC programming.

## Overview

This crate provides fundamental type definitions used across PLC programming tools:

- **POU Types**: Program, Function, FunctionBlock
- **Variable Classes**: Input, Output, InOut, Local, Temp, External, Global
- **Data Types**: Elementary types (BOOL, INT, REAL, etc.)

## Usage

```rust
use iectypes::{PouType, VarClass, DataType};

let pou_type = PouType::FunctionBlock;
let var_class = VarClass::Input;
```

## Types

### PouType
Program Organization Unit types per IEC 61131-3:
- `Program` - Scheduled by tasks, has state
- `Function` - Stateless, returns a value
- `FunctionBlock` - Stateful, instantiated

### VarClass
Variable classification:
- `Input`, `Output`, `InOut` - Interface variables
- `Local`, `Temp` - Internal variables
- `External`, `Global` - Shared variables
- `Constant` - Compile-time constants

### DataType
Elementary IEC 61131-3 data types:
- Boolean: `BOOL`
- Integer: `SINT`, `INT`, `DINT`, `LINT`, `USINT`, `UINT`, `UDINT`, `ULINT`
- Real: `REAL`, `LREAL`
- String: `STRING`, `WSTRING`
- Time: `TIME`, `DATE`, `TOD`, `DT`
- Bit: `BYTE`, `WORD`, `DWORD`, `LWORD`

## License

MIT
