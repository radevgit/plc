//! IEC 61131-3 Elementary Data Types
//!
//! Reference: IEC 61131-3, Table 10 - Elementary data types

use std::fmt;

/// Elementary data types as defined in IEC 61131-3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    // Boolean
    /// Boolean (1 bit)
    Bool,

    // Integer types (signed)
    /// Signed 8-bit integer (-128 to 127)
    SInt,
    /// Signed 16-bit integer (-32768 to 32767)
    Int,
    /// Signed 32-bit integer
    DInt,
    /// Signed 64-bit integer
    LInt,

    // Integer types (unsigned)
    /// Unsigned 8-bit integer (0 to 255)
    USInt,
    /// Unsigned 16-bit integer (0 to 65535)
    UInt,
    /// Unsigned 32-bit integer
    UDInt,
    /// Unsigned 64-bit integer
    ULInt,

    // Floating point
    /// 32-bit floating point (IEEE 754)
    Real,
    /// 64-bit floating point (IEEE 754)
    LReal,

    // Duration
    /// Time duration
    Time,
    /// Long time duration (64-bit)
    LTime,

    // Date and time
    /// Date (year, month, day)
    Date,
    /// Long date
    LDate,
    /// Time of day
    TimeOfDay,
    /// Long time of day
    LTimeOfDay,
    /// Date and time combined
    DateTime,
    /// Long date and time
    LDateTime,

    // String types
    /// Single-byte character string
    String,
    /// Wide (Unicode) character string
    WString,
    /// Single character
    Char,
    /// Wide character
    WChar,

    // Bit string types
    /// 8-bit bit string
    Byte,
    /// 16-bit bit string
    Word,
    /// 32-bit bit string
    DWord,
    /// 64-bit bit string
    LWord,
}

impl DataType {
    /// Get the size in bits for this data type
    pub fn size_bits(&self) -> usize {
        match self {
            DataType::Bool => 1,

            DataType::SInt | DataType::USInt | DataType::Byte | DataType::Char => 8,

            DataType::Int | DataType::UInt | DataType::Word | DataType::WChar => 16,

            DataType::DInt
            | DataType::UDInt
            | DataType::DWord
            | DataType::Real
            | DataType::Time
            | DataType::Date
            | DataType::TimeOfDay => 32,

            DataType::LInt
            | DataType::ULInt
            | DataType::LWord
            | DataType::LReal
            | DataType::LTime
            | DataType::LDate
            | DataType::LTimeOfDay
            | DataType::DateTime
            | DataType::LDateTime => 64,

            // Variable size - return 0 or a default
            DataType::String | DataType::WString => 0,
        }
    }

    /// Get the size in bytes for this data type
    pub fn size_bytes(&self) -> usize {
        let bits = self.size_bits();
        if bits == 0 {
            0
        } else {
            (bits + 7) / 8
        }
    }

    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            DataType::SInt
                | DataType::Int
                | DataType::DInt
                | DataType::LInt
                | DataType::USInt
                | DataType::UInt
                | DataType::UDInt
                | DataType::ULInt
                | DataType::Real
                | DataType::LReal
        )
    }

    /// Check if this is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            DataType::SInt
                | DataType::Int
                | DataType::DInt
                | DataType::LInt
                | DataType::USInt
                | DataType::UInt
                | DataType::UDInt
                | DataType::ULInt
        )
    }

    /// Check if this is a signed integer type
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            DataType::SInt | DataType::Int | DataType::DInt | DataType::LInt
        )
    }

    /// Check if this is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, DataType::Real | DataType::LReal)
    }

    /// Check if this is a bit string type
    pub fn is_bit_string(&self) -> bool {
        matches!(
            self,
            DataType::Bool | DataType::Byte | DataType::Word | DataType::DWord | DataType::LWord
        )
    }

    /// Check if this is a time-related type
    pub fn is_time(&self) -> bool {
        matches!(
            self,
            DataType::Time
                | DataType::LTime
                | DataType::Date
                | DataType::LDate
                | DataType::TimeOfDay
                | DataType::LTimeOfDay
                | DataType::DateTime
                | DataType::LDateTime
        )
    }

    /// Check if this is a string type
    pub fn is_string(&self) -> bool {
        matches!(
            self,
            DataType::String | DataType::WString | DataType::Char | DataType::WChar
        )
    }

    /// Parse a data type from its IEC name (case-insensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "BOOL" => Some(DataType::Bool),

            "SINT" => Some(DataType::SInt),
            "INT" => Some(DataType::Int),
            "DINT" => Some(DataType::DInt),
            "LINT" => Some(DataType::LInt),

            "USINT" => Some(DataType::USInt),
            "UINT" => Some(DataType::UInt),
            "UDINT" => Some(DataType::UDInt),
            "ULINT" => Some(DataType::ULInt),

            "REAL" => Some(DataType::Real),
            "LREAL" => Some(DataType::LReal),

            "TIME" => Some(DataType::Time),
            "LTIME" => Some(DataType::LTime),
            "DATE" => Some(DataType::Date),
            "LDATE" => Some(DataType::LDate),
            "TIME_OF_DAY" | "TOD" => Some(DataType::TimeOfDay),
            "LTIME_OF_DAY" | "LTOD" => Some(DataType::LTimeOfDay),
            "DATE_AND_TIME" | "DT" => Some(DataType::DateTime),
            "LDATE_AND_TIME" | "LDT" => Some(DataType::LDateTime),

            "STRING" => Some(DataType::String),
            "WSTRING" => Some(DataType::WString),
            "CHAR" => Some(DataType::Char),
            "WCHAR" => Some(DataType::WChar),

            "BYTE" => Some(DataType::Byte),
            "WORD" => Some(DataType::Word),
            "DWORD" => Some(DataType::DWord),
            "LWORD" => Some(DataType::LWord),

            _ => None,
        }
    }

    /// Get the IEC name of this data type
    pub fn name(&self) -> &'static str {
        match self {
            DataType::Bool => "BOOL",

            DataType::SInt => "SINT",
            DataType::Int => "INT",
            DataType::DInt => "DINT",
            DataType::LInt => "LINT",

            DataType::USInt => "USINT",
            DataType::UInt => "UINT",
            DataType::UDInt => "UDINT",
            DataType::ULInt => "ULINT",

            DataType::Real => "REAL",
            DataType::LReal => "LREAL",

            DataType::Time => "TIME",
            DataType::LTime => "LTIME",
            DataType::Date => "DATE",
            DataType::LDate => "LDATE",
            DataType::TimeOfDay => "TIME_OF_DAY",
            DataType::LTimeOfDay => "LTIME_OF_DAY",
            DataType::DateTime => "DATE_AND_TIME",
            DataType::LDateTime => "LDATE_AND_TIME",

            DataType::String => "STRING",
            DataType::WString => "WSTRING",
            DataType::Char => "CHAR",
            DataType::WChar => "WCHAR",

            DataType::Byte => "BYTE",
            DataType::Word => "WORD",
            DataType::DWord => "DWORD",
            DataType::LWord => "LWORD",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Variable scope/class as defined in IEC 61131-3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarClass {
    /// Local variables (VAR)
    Local,
    /// Input variables (VAR_INPUT)
    Input,
    /// Output variables (VAR_OUTPUT)
    Output,
    /// In/Out variables (VAR_IN_OUT)
    InOut,
    /// External variables (VAR_EXTERNAL)
    External,
    /// Global variables (VAR_GLOBAL)
    Global,
    /// Temporary variables (VAR_TEMP)
    Temp,
    /// Configuration variables (VAR_CONFIG)
    Config,
    /// Access variables (VAR_ACCESS)
    Access,
}

impl VarClass {
    /// Parse from IEC keyword
    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw.to_uppercase().as_str() {
            "VAR" => Some(VarClass::Local),
            "VAR_INPUT" => Some(VarClass::Input),
            "VAR_OUTPUT" => Some(VarClass::Output),
            "VAR_IN_OUT" => Some(VarClass::InOut),
            "VAR_EXTERNAL" => Some(VarClass::External),
            "VAR_GLOBAL" => Some(VarClass::Global),
            "VAR_TEMP" => Some(VarClass::Temp),
            "VAR_CONFIG" => Some(VarClass::Config),
            "VAR_ACCESS" => Some(VarClass::Access),
            _ => None,
        }
    }

    /// Get the IEC keyword
    pub fn keyword(&self) -> &'static str {
        match self {
            VarClass::Local => "VAR",
            VarClass::Input => "VAR_INPUT",
            VarClass::Output => "VAR_OUTPUT",
            VarClass::InOut => "VAR_IN_OUT",
            VarClass::External => "VAR_EXTERNAL",
            VarClass::Global => "VAR_GLOBAL",
            VarClass::Temp => "VAR_TEMP",
            VarClass::Config => "VAR_CONFIG",
            VarClass::Access => "VAR_ACCESS",
        }
    }
}

impl fmt::Display for VarClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.keyword())
    }
}

/// POU (Program Organization Unit) type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PouType {
    /// PROGRAM
    Program,
    /// FUNCTION
    Function,
    /// FUNCTION_BLOCK
    FunctionBlock,
}

impl PouType {
    pub fn from_keyword(kw: &str) -> Option<Self> {
        match kw.to_uppercase().as_str() {
            "PROGRAM" => Some(PouType::Program),
            "FUNCTION" => Some(PouType::Function),
            "FUNCTION_BLOCK" | "FB" => Some(PouType::FunctionBlock),
            _ => None,
        }
    }

    pub fn keyword(&self) -> &'static str {
        match self {
            PouType::Program => "PROGRAM",
            PouType::Function => "FUNCTION",
            PouType::FunctionBlock => "FUNCTION_BLOCK",
        }
    }
}

impl fmt::Display for PouType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.keyword())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_sizes() {
        assert_eq!(DataType::Bool.size_bits(), 1);
        assert_eq!(DataType::Int.size_bits(), 16);
        assert_eq!(DataType::DInt.size_bits(), 32);
        assert_eq!(DataType::LReal.size_bits(), 64);
    }

    #[test]
    fn test_data_type_from_name() {
        assert_eq!(DataType::from_name("INT"), Some(DataType::Int));
        assert_eq!(DataType::from_name("int"), Some(DataType::Int));
        assert_eq!(DataType::from_name("DINT"), Some(DataType::DInt));
        assert_eq!(DataType::from_name("TOD"), Some(DataType::TimeOfDay));
        assert_eq!(DataType::from_name("UNKNOWN"), None);
    }

    #[test]
    fn test_type_categories() {
        assert!(DataType::Int.is_integer());
        assert!(DataType::Int.is_numeric());
        assert!(DataType::Int.is_signed());

        assert!(DataType::UInt.is_integer());
        assert!(!DataType::UInt.is_signed());

        assert!(DataType::Real.is_float());
        assert!(DataType::Real.is_numeric());
        assert!(!DataType::Real.is_integer());

        assert!(DataType::Time.is_time());
        assert!(DataType::Word.is_bit_string());
        assert!(DataType::String.is_string());
    }

    #[test]
    fn test_var_class() {
        assert_eq!(VarClass::from_keyword("VAR_INPUT"), Some(VarClass::Input));
        assert_eq!(VarClass::Input.keyword(), "VAR_INPUT");
    }
}
