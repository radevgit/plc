//! User-defined data type definitions.

/// A user-defined data type.
///
/// Maps to:
/// - L5X: `<DataType>` elements
/// - PLCopen: `<dataType>` within `<types>/<dataTypes>`
#[derive(Debug, Clone)]
pub struct DataTypeDef {
    /// Type name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// The actual type definition
    pub kind: DataTypeKind,
}

/// The kind of user-defined type.
#[derive(Debug, Clone)]
pub enum DataTypeKind {
    /// Alias to another type
    Alias(String),

    /// Structure with named members
    Struct(StructDef),

    /// Enumeration
    Enum(EnumDef),

    /// Array type
    Array(ArrayDef),

    /// Subrange type (INTEGER 0..100)
    Subrange {
        base_type: String,
        lower: i64,
        upper: i64,
    },
}

impl DataTypeDef {
    /// Create a struct type.
    pub fn structure(name: impl Into<String>, members: Vec<StructMember>) -> Self {
        Self {
            name: name.into(),
            description: None,
            kind: DataTypeKind::Struct(StructDef { members }),
        }
    }

    /// Create an enum type.
    pub fn enumeration(name: impl Into<String>, members: Vec<EnumMember>) -> Self {
        Self {
            name: name.into(),
            description: None,
            kind: DataTypeKind::Enum(EnumDef {
                base_type: None,
                members,
            }),
        }
    }

    /// Create an alias type.
    pub fn alias(name: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            kind: DataTypeKind::Alias(target.into()),
        }
    }
}

/// Structure definition.
#[derive(Debug, Clone, Default)]
pub struct StructDef {
    /// Structure members
    pub members: Vec<StructMember>,
}

/// A member of a structure.
#[derive(Debug, Clone)]
pub struct StructMember {
    /// Member name
    pub name: String,

    /// Data type
    pub data_type: String,

    /// Initial value
    pub initial_value: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Array dimensions (empty for scalar)
    pub dimensions: Vec<u32>,
}

impl StructMember {
    /// Create a simple struct member.
    pub fn new(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            initial_value: None,
            description: None,
            dimensions: Vec::new(),
        }
    }
}

/// Enumeration definition.
#[derive(Debug, Clone, Default)]
pub struct EnumDef {
    /// Base type (default DINT for most PLCs)
    pub base_type: Option<String>,

    /// Enumeration members
    pub members: Vec<EnumMember>,
}

/// A member of an enumeration.
#[derive(Debug, Clone)]
pub struct EnumMember {
    /// Member name
    pub name: String,

    /// Explicit value (if specified)
    pub value: Option<i64>,

    /// Description
    pub description: Option<String>,
}

impl EnumMember {
    /// Create an enum member.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            description: None,
        }
    }

    /// Create an enum member with explicit value.
    pub fn with_value(name: impl Into<String>, value: i64) -> Self {
        Self {
            name: name.into(),
            value: Some(value),
            description: None,
        }
    }
}

/// Array type definition.
#[derive(Debug, Clone)]
pub struct ArrayDef {
    /// Element type
    pub element_type: String,

    /// Dimensions with bounds
    pub dimensions: Vec<ArrayDimension>,
}

/// An array dimension with lower and upper bounds.
#[derive(Debug, Clone)]
pub struct ArrayDimension {
    /// Lower bound (usually 0 or 1)
    pub lower: i32,

    /// Upper bound
    pub upper: i32,
}

impl ArrayDimension {
    /// Create a zero-based dimension [0..size-1]
    pub fn zero_based(size: u32) -> Self {
        Self {
            lower: 0,
            upper: size as i32 - 1,
        }
    }

    /// Create a one-based dimension [1..size]
    pub fn one_based(size: u32) -> Self {
        Self {
            lower: 1,
            upper: size as i32,
        }
    }

    /// Get the size of this dimension.
    pub fn size(&self) -> u32 {
        (self.upper - self.lower + 1) as u32
    }
}
