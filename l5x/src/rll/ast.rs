//! AST types for RLL (Relay Ladder Logic).

use crate::rll::error::RllError;

/// A parsed rung of ladder logic.
#[derive(Debug, Clone, PartialEq)]
pub struct Rung {
    /// Original text (for round-trip/debugging)
    pub raw_text: String,
    /// Parsed content (None if parsing failed)
    pub content: Option<RungContent>,
    /// Parse error if parsing failed
    pub error: Option<RllError>,
}

impl Rung {
    /// Create a successfully parsed rung
    pub fn ok(raw_text: String, content: RungContent) -> Self {
        Self {
            raw_text,
            content: Some(content),
            error: None,
        }
    }

    /// Create a rung that failed to parse (permissive mode)
    pub fn err(raw_text: String, error: RllError) -> Self {
        Self {
            raw_text,
            content: None,
            error: Some(error),
        }
    }

    /// Returns true if this rung was successfully parsed
    pub fn is_parsed(&self) -> bool {
        self.content.is_some()
    }

    /// Extract all tag references from this rung
    pub fn tag_references(&self) -> Vec<TagReference> {
        match &self.content {
            Some(content) => content.tag_references(),
            None => Vec::new(),
        }
    }
}

/// Successfully parsed rung content.
#[derive(Debug, Clone, PartialEq)]
pub struct RungContent {
    /// The elements in this rung (instructions and parallel branches)
    pub elements: Vec<RungElement>,
}

impl RungContent {
    /// Create new rung content
    pub fn new(elements: Vec<RungElement>) -> Self {
        Self { elements }
    }

    /// Extract all tag references from this rung content
    pub fn tag_references(&self) -> Vec<TagReference> {
        let mut refs = Vec::new();
        for element in &self.elements {
            element.collect_tag_references(&mut refs);
        }
        refs
    }
}

/// Element in a rung (instruction or parallel branch).
#[derive(Debug, Clone, PartialEq)]
pub enum RungElement {
    /// Single instruction: XIC(tag), MOV(src,dest), etc.
    Instruction(Instruction),
    /// Parallel branches: [branch1, branch2, ...]
    Parallel(Vec<Branch>),
}

impl RungElement {
    /// Collect tag references from this element
    fn collect_tag_references(&self, refs: &mut Vec<TagReference>) {
        match self {
            RungElement::Instruction(instr) => {
                instr.collect_tag_references(refs);
            }
            RungElement::Parallel(branches) => {
                for branch in branches {
                    for element in &branch.elements {
                        element.collect_tag_references(refs);
                    }
                }
            }
        }
    }
}

/// A branch within a parallel structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    /// Elements in this branch
    pub elements: Vec<RungElement>,
}

impl Branch {
    /// Create a new branch
    pub fn new(elements: Vec<RungElement>) -> Self {
        Self { elements }
    }
}

/// A single instruction call.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// Instruction mnemonic: "XIC", "TON", "MOV", etc.
    pub mnemonic: String,
    /// Operands (tags, literals, or inferred)
    pub operands: Vec<Operand>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(mnemonic: impl Into<String>, operands: Vec<Operand>) -> Self {
        Self {
            mnemonic: mnemonic.into(),
            operands,
        }
    }

    /// Collect tag references from this instruction's operands
    fn collect_tag_references(&self, refs: &mut Vec<TagReference>) {
        for (index, operand) in self.operands.iter().enumerate() {
            if let Operand::Value(value) = operand {
                // Parse the operand to extract actual tag references
                let parsed = crate::rll::operand::parse_operand_value(value);
                for tag in parsed.all_tags() {
                    refs.push(TagReference {
                        name: tag,
                        full_operand: value.clone(),
                        instruction: self.mnemonic.clone(),
                        operand_index: index,
                    });
                }
            }
        }
    }
}

/// An operand to an instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Inferred/default operand: ?
    Inferred,
    /// Explicit operand value (tag reference, literal, or expression)
    Value(String),
}

impl Operand {
    /// Create an inferred operand
    pub fn inferred() -> Self {
        Self::Inferred
    }

    /// Create a value operand
    pub fn value(s: impl Into<String>) -> Self {
        Self::Value(s.into())
    }

    /// Returns true if this is an inferred operand
    pub fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred)
    }

    /// Get the value if this is a Value operand
    pub fn as_value(&self) -> Option<&str> {
        match self {
            Self::Value(v) => Some(v),
            Self::Inferred => None,
        }
    }
}

/// A reference to a tag found in a rung.
#[derive(Debug, Clone, PartialEq)]
pub struct TagReference {
    /// The base tag name (e.g., "Timer1" from "Timer1.PRE")
    pub name: String,
    /// The full operand as written (e.g., "Timer1.PRE" or "Array[idx].Member")
    pub full_operand: String,
    /// The instruction that references this tag
    pub instruction: String,
    /// Which operand position (0-indexed)
    pub operand_index: usize,
}
