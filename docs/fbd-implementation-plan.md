# FBD Reference Extraction Implementation Plan

## Problem Statement

Currently, the PLCopen analyzer in `plceye` **skips** FBD (Function Block Diagram) analysis entirely. When encountering FBD bodies, it only increments a counter but doesn't extract:
- Variable references (input/output/inOut variables)
- Function block calls (block instances)
- POU calls (function/FB invocations)

This means FBD-based programs are **invisible** to analysis tools like unused variable detection and cross-reference building.

**Current Code Location:** `plceye/src/analysis/plcopen_analysis.rs:239`

```rust
// FBD body
if body.fbd.is_some() {
    analysis.stats.fbd_bodies += 1;
    has_code = true;
    // TODO: Extract references from FBD elements
}
```

---

## Background: FBD Structure in PLCopen

### FBD Elements

```rust
pub enum FbdObjects {
    Block(FbdObjects_block_Inline),              // FB/Function instances
    InVariable(FbdObjects_inVariable_Inline),    // Variable reads
    OutVariable(FbdObjects_outVariable_Inline),  // Variable writes
    InOutVariable(FbdObjects_inOutVariable_Inline), // In/out variables
    Label(FbdObjects_label_Inline),              // Jump labels
    Jump(FbdObjects_jump_Inline),                // Jump statements
    Return(FbdObjects_return_Inline),            // Return statements
}
```

### Key Reference Sources

1. **Block instances** (`typeName` attribute) - Function/FB calls
2. **InVariable** (`expression` field) - Variable reads (e.g., `IN1`, `MyVar`)
3. **OutVariable** (`expression` field) - Variable writes
4. **InOutVariable** (`expression` field) - Bidirectional access
5. **Block parameters** (inputVariables, outputVariables, inOutVariables)

### Example XML

```xml
<FBD>
    <inVariable localId="2">
        <expression>IN1</expression>  <!-- Variable reference! -->
    </inVariable>
    <block localId="6" typeName="AND">  <!-- POU call! -->
        <inputVariables>
            <variable formalParameter="IN1">
                <connectionPointIn refLocalId="2"/>
            </variable>
        </inputVariables>
    </block>
    <outVariable localId="4">
        <expression>FBDTest</expression>  <!-- Variable reference! -->
    </outVariable>
</FBD>
```

---

## Root Cause: XSD Schema Issue

**Problem:** `Body_FBD_Inline` is currently **empty**:

```rust
pub struct Body_FBD_Inline {
    // Empty! No children captured!
}
```

The PLCopen schema defines FBD as:

```xml
<xsd:element name="FBD">
    <xsd:complexType>
        <xsd:choice minOccurs="0" maxOccurs="unbounded">
            <xsd:group ref="ppx:commonObjects"/>
            <xsd:group ref="ppx:fbdObjects"/>
        </xsd:choice>
    </xsd:complexType>
</xsd:element>
```

The code generator needs to handle **unbounded choice groups** properly. Currently generates empty struct.

---

## Implementation Plan

### Phase 1: Fix Code Generation (CRITICAL)

**Goal:** Make `Body_FBD_Inline` actually contain FBD elements

**Files to modify:**
- `plcopen/build/xsd_parser.rs`
- `plcopen/build/codegen.rs`

**Steps:**

1. **Update XSD parser**
   - Detect `<xsd:choice maxOccurs="unbounded">` patterns
   - Mark these as needing Vec collections

2. **Update code generator**
   - For unbounded choices, generate:
   
   ```rust
   pub struct Body_FBD_Inline {
       #[serde(rename = "$value")]
       pub elements: Vec<FbdElement>,
   }
   
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   #[serde(untagged)]
   pub enum FbdElement {
       // CommonObjects variants
       Comment(CommonObjects_comment_Inline),
       Connector(CommonObjects_connector_Inline),
       Continuation(CommonObjects_continuation_Inline),
       // FbdObjects variants
       Block(FbdObjects_block_Inline),
       InVariable(FbdObjects_inVariable_Inline),
       OutVariable(FbdObjects_outVariable_Inline),
       InOutVariable(FbdObjects_inOutVariable_Inline),
       Label(FbdObjects_label_Inline),
       Jump(FbdObjects_jump_Inline),
       Return(FbdObjects_return_Inline),
   }
   ```

3. **Test with real files**
   - Parse `~/devpublic/dataplc/plcopen/example1.xml`
   - Verify elements are captured
   - Write unit test

**Time estimate:** 1-2 days

**Success criteria:**
- [ ] `Body_FBD_Inline` contains `elements: Vec<FbdElement>`
- [ ] Deserialization populates the elements vector
- [ ] Test files parse without errors
- [ ] All FBD element types are captured

---

### Phase 2: Implement Reference Extraction

**Goal:** Extract all variable and POU references from FBD

**File:** `plceye/src/analysis/plcopen_analysis.rs`

**Steps:**

1. **Create extraction function:**

```rust
fn extract_references_from_fbd(
    fbd: &Body_FBD_Inline,
    analysis: &mut PlcopenAnalysis
) {
    for element in &fbd.elements {
        match element {
            FbdElement::InVariable(var) => {
                if let Some(expr) = &var.expression {
                    let base_var = extract_base_variable(expr);
                    analysis.used_variables.insert(base_var);
                }
            }
            FbdElement::OutVariable(var) => {
                if let Some(expr) = &var.expression {
                    let base_var = extract_base_variable(expr);
                    analysis.used_variables.insert(base_var);
                }
            }
            FbdElement::InOutVariable(var) => {
                if let Some(expr) = &var.expression {
                    let base_var = extract_base_variable(expr);
                    analysis.used_variables.insert(base_var);
                }
            }
            FbdElement::Block(block) => {
                // Extract typeName as POU call
                analysis.used_variables.insert(block.type_name.clone());
                
                // Extract variable connections from parameters
                extract_block_parameters(block, analysis);
            }
            _ => {} // Handle other types as needed
        }
    }
}

fn extract_block_parameters(
    block: &FbdObjects_block_Inline,
    analysis: &mut PlcopenAnalysis
) {
    // Extract from inputVariables
    if let Some(inputs) = &block.input_variables {
        for var in &inputs.variable {
            // Variables can be referenced in connection metadata
            // or through instance names
            if let Some(ref conn) = var.connection_point_in {
                // Extract referenced variables from connections
                // This requires analyzing connectionPointIn structure
            }
        }
    }
    
    // Similar for outputVariables
    if let Some(outputs) = &block.output_variables {
        for var in &outputs.variable {
            // Process output connections
        }
    }
    
    // Similar for inOutVariables
    if let Some(inouts) = &block.in_out_variables {
        for var in &inouts.variable {
            // Process inout connections
        }
    }
}

fn extract_base_variable(expression: &str) -> String {
    // Parse "MyArray[5].Field" -> "MyArray"
    // Parse "MyStruct.Field" -> "MyStruct"
    expression
        .split(|c| c == '[' || c == '.')
        .next()
        .unwrap_or(expression)
        .trim()
        .to_string()
}
```

2. **Update body analysis:**

```rust
// FBD body
if let Some(fbd) = &body.fbd {
    analysis.stats.fbd_bodies += 1;
    has_code = true;
    extract_references_from_fbd(fbd, analysis);  // NEW!
}
```

3. **Add tests:**

```rust
#[test]
fn test_extract_fbd_references() {
    let xml = r#"<?xml version="1.0"?>
    <project>
        <types>
            <pous>
                <pou name="TestPOU" pouType="functionBlock">
                    <body>
                        <FBD>
                            <inVariable localId="1">
                                <expression>MyInput</expression>
                            </inVariable>
                            <block localId="2" typeName="TON">
                                <inputVariables>
                                    <variable formalParameter="IN">
                                        <connectionPointIn>
                                            <connection refLocalId="1"/>
                                        </connectionPointIn>
                                    </variable>
                                </inputVariables>
                            </block>
                            <outVariable localId="3">
                                <expression>MyOutput</expression>
                            </outVariable>
                        </FBD>
                    </body>
                </pou>
            </pous>
        </types>
    </project>
    "#;
    
    let project: plcopen::Project = plcopen::from_str(xml).unwrap();
    let analysis = analyze_plcopen_project(&project);
    
    assert!(analysis.used_variables.contains("MyInput"));
    assert!(analysis.used_variables.contains("MyOutput"));
    assert!(analysis.used_variables.contains("TON"));
}

#[test]
fn test_extract_complex_fbd_expressions() {
    // Test array access: MyArray[5]
    assert_eq!(extract_base_variable("MyArray[5]"), "MyArray");
    
    // Test struct access: MyStruct.Field
    assert_eq!(extract_base_variable("MyStruct.Field"), "MyStruct");
    
    // Test complex: Motors[i].Speed
    assert_eq!(extract_base_variable("Motors[i].Speed"), "Motors");
}
```

**Time estimate:** 2-3 days

**Success criteria:**
- [ ] Variable references extracted from inVariable/outVariable/inOutVariable
- [ ] POU calls extracted from block typeName
- [ ] Complex expressions parsed correctly (arrays, structs)
- [ ] Unit tests pass
- [ ] Integration with PlcopenAnalysis works

---

### Phase 3: Handle Edge Cases

**Goal:** Handle special cases and improve robustness

**Cases to handle:**

1. **Literal expressions** - Some inVariable elements might contain literals instead of variable names
   ```xml
   <inVariable localId="5">
       <expression>100</expression>  <!-- Constant, not a variable -->
   </inVariable>
   ```

2. **Empty expressions** - Handle missing or empty expression fields

3. **Instance names** - Track FB instance names from blocks:
   ```xml
   <block localId="6" typeName="TON" instanceName="MyTimer">
   ```

4. **Execution order** - Optional: respect executionOrderId for analysis order

**Implementation:**

```rust
fn is_variable_name(expr: &str) -> bool {
    // Check if expression looks like a variable name
    // vs a literal (number, quoted string, etc.)
    let trimmed = expr.trim();
    
    // Skip empty
    if trimmed.is_empty() {
        return false;
    }
    
    // Skip numeric literals
    if trimmed.parse::<f64>().is_ok() {
        return false;
    }
    
    // Skip quoted strings
    if trimmed.starts_with('\'') || trimmed.starts_with('"') {
        return false;
    }
    
    // Must start with letter or underscore
    trimmed.chars().next()
        .map(|c| c.is_alphabetic() || c == '_')
        .unwrap_or(false)
}

fn extract_references_from_fbd(
    fbd: &Body_FBD_Inline,
    analysis: &mut PlcopenAnalysis
) {
    for element in &fbd.elements {
        match element {
            FbdElement::InVariable(var) => {
                if let Some(expr) = &var.expression {
                    if is_variable_name(expr) {  // NEW: Filter literals
                        let base_var = extract_base_variable(expr);
                        analysis.used_variables.insert(base_var);
                    }
                }
            }
            FbdElement::Block(block) => {
                // Track POU call
                analysis.used_variables.insert(block.type_name.clone());
                
                // Track instance name if present
                if let Some(ref instance) = block.instance_name {
                    analysis.defined_variables.insert(instance.clone());
                }
                
                extract_block_parameters(block, analysis);
            }
            // ... other cases
        }
    }
}
```

**Time estimate:** 1 day

**Success criteria:**
- [ ] Literals filtered out (not treated as variables)
- [ ] Empty expressions handled gracefully
- [ ] Instance names tracked as variable definitions
- [ ] Edge case tests pass

---

### Phase 4: Integration & Testing

**Goal:** Ensure FBD analysis works end-to-end in plceye

**Steps:**

1. **Test with real PLCopen projects**
   - Use files from `~/devpublic/dataplc/plcopen/`
   - Verify unused variable detection works for FBD
   - Verify cross-references are built correctly

2. **Add integration tests:**

```rust
#[test]
fn test_fbd_unused_detection() {
    let xml = include_str!("../../test_data/fbd_with_unused.xml");
    let project: plcopen::Project = plcopen::from_str(xml).unwrap();
    let analysis = analyze_plcopen_project(&project);
    
    // Variable used in FBD should not be flagged
    assert!(!analysis.is_unused("UsedInFBD"));
    
    // Variable declared but never used should be flagged
    assert!(analysis.is_unused("DeclaredButUnused"));
}

#[test]
fn test_fbd_cross_references() {
    let xml = include_str!("../../test_data/fbd_references.xml");
    let project: plcopen::Project = plcopen::from_str(xml).unwrap();
    let analysis = analyze_plcopen_project(&project);
    
    // Check that TON timer is tracked as used
    let refs = analysis.get_references("TON");
    assert!(!refs.is_empty());
    assert_eq!(refs[0].pou_name, "Main");
}
```

3. **Update plceye detector rules**
   - Ensure FBD-referenced variables exempt from unused detection
   - Ensure FBD POUs tracked in cross-references

4. **Performance testing**
   - Test with large FBD projects
   - Ensure no performance degradation

5. **Update documentation**
   - Document FBD support in README
   - Add examples showing FBD analysis
   - Update CHANGELOG

**Time estimate:** 1-2 days

**Success criteria:**
- [ ] Real PLCopen files with FBD parse and analyze correctly
- [ ] Unused variable detector respects FBD references
- [ ] Cross-reference building includes FBD calls
- [ ] No performance regressions
- [ ] Documentation updated

---

## Similar Work: LD and SFC

After FBD is complete, the **same pattern** applies to:

### Ladder Diagram (LD)

```rust
pub enum LdObjects {
    LeftPowerRail(LdObjects_leftPowerRail_Inline),
    RightPowerRail(LdObjects_rightPowerRail_Inline),
    Coil(LdObjects_coil_Inline),              // Variable writes
    Contact(LdObjects_contact_Inline),        // Variable reads
}
```

Key references:
- `Contact.variable` - variable reads
- `Coil.variable` - variable writes
- Block calls (if supported in LD)

### Sequential Function Chart (SFC)

```rust
pub enum SfcObjects {
    Step(SfcObjects_step_Inline),
    Transition(SfcObjects_transition_Inline),
    Action(SfcObjects_action_Inline),
    // ...
}
```

Key references:
- Actions contain embedded code (ST, FBD, LD)
- Transitions contain conditions
- Step names are identifiers

---

## Summary

### Total Time Estimate: **5-8 days**

### Phases Breakdown:
- **Phase 1:** Fix codegen (1-2 days) - **CRITICAL BLOCKER**
- **Phase 2:** Reference extraction (2-3 days)
- **Phase 3:** Edge cases (1 day)
- **Phase 4:** Integration (1-2 days)

### Priority: **High**

**Rationale:**
- FBD is widely used in industrial automation (IEC 61131-3 standard)
- Missing FBD analysis means incomplete code coverage
- Directly affects accuracy of:
  - Unused variable detection
  - Unused POU detection
  - Cross-reference generation
  - Dependency analysis

### Dependencies:
- None (self-contained work)
- Can be done in parallel with LD/SFC if desired (same pattern)

### Risks:
- **Medium:** XSD parser changes might affect other types
  - **Mitigation:** Comprehensive testing with existing test suite
- **Low:** Performance impact from parsing complex FBD networks
  - **Mitigation:** Benchmark with large files

### Success Metrics:
1. ✅ FBD bodies deserialized with all elements
2. ✅ Variable references extracted from inVariable/outVariable/inOutVariable
3. ✅ POU calls extracted from block typeName
4. ✅ Tests pass with real PLCopen files
5. ✅ Integration with unused variable detector works
6. ✅ No false positives/negatives in analysis results

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Start Phase 1** - Fix code generation for FBD elements
3. **Create test files** - Prepare PLCopen XML samples for testing
4. **Set up benchmark** - Baseline performance before changes

## Notes

- This work will also benefit LD and SFC analysis (same pattern)
- Consider creating a helper module for graphical language analysis
- Document the XSD parser changes for future reference
