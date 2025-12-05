//! IEC 61131-3 Literal Parsing
//!
//! Parses IEC literal formats:
//! - Time: T#1d2h3m4s5ms, TIME#500ms
//! - Numeric: 16#FF, 2#1010, 8#77
//! - Typed: INT#100, REAL#3.14

use std::fmt;

/// Time duration literal (T#, TIME#)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeLiteral {
    /// Days
    pub days: u32,
    /// Hours
    pub hours: u32,
    /// Minutes
    pub minutes: u32,
    /// Seconds
    pub seconds: u32,
    /// Milliseconds
    pub milliseconds: u32,
    /// Microseconds
    pub microseconds: u32,
    /// Nanoseconds
    pub nanoseconds: u32,
    /// Negative duration
    pub negative: bool,
}

impl TimeLiteral {
    /// Create a new zero duration
    pub fn zero() -> Self {
        Self::default()
    }

    /// Create from milliseconds
    pub fn from_milliseconds(ms: i64) -> Self {
        let negative = ms < 0;
        let ms = ms.unsigned_abs();

        let days = (ms / (24 * 60 * 60 * 1000)) as u32;
        let remaining = ms % (24 * 60 * 60 * 1000);
        let hours = (remaining / (60 * 60 * 1000)) as u32;
        let remaining = remaining % (60 * 60 * 1000);
        let minutes = (remaining / (60 * 1000)) as u32;
        let remaining = remaining % (60 * 1000);
        let seconds = (remaining / 1000) as u32;
        let milliseconds = (remaining % 1000) as u32;

        Self {
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds: 0,
            nanoseconds: 0,
            negative,
        }
    }

    /// Convert to total milliseconds
    pub fn to_milliseconds(&self) -> i64 {
        let ms = self.days as i64 * 24 * 60 * 60 * 1000
            + self.hours as i64 * 60 * 60 * 1000
            + self.minutes as i64 * 60 * 1000
            + self.seconds as i64 * 1000
            + self.milliseconds as i64;

        if self.negative { -ms } else { ms }
    }

    /// Convert to total microseconds
    pub fn to_microseconds(&self) -> i64 {
        self.to_milliseconds() * 1000 + self.microseconds as i64
    }

    /// Convert to total nanoseconds
    pub fn to_nanoseconds(&self) -> i128 {
        self.to_microseconds() as i128 * 1000 + self.nanoseconds as i128
    }

    /// Parse a time literal string (T#1s, TIME#500ms, etc.)
    pub fn parse(s: &str) -> Result<Self, LiteralError> {
        let s = s.trim();

        // Check prefix
        let content = if let Some(rest) = s.strip_prefix("T#") {
            rest
        } else if let Some(rest) = s.strip_prefix("TIME#") {
            rest
        } else if let Some(rest) = s.strip_prefix("t#") {
            rest
        } else if let Some(rest) = s.strip_prefix("time#") {
            rest
        } else {
            return Err(LiteralError::InvalidPrefix("T# or TIME#"));
        };

        // Check for negative
        let (negative, content) = if let Some(rest) = content.strip_prefix('-') {
            (true, rest)
        } else {
            (false, content)
        };

        let mut result = TimeLiteral {
            negative,
            ..Default::default()
        };

        // Parse components: 1d2h3m4s5ms6us7ns
        let mut chars = content.chars().peekable();
        let mut num_str = String::new();

        while let Some(c) = chars.next() {
            if c.is_ascii_digit() || c == '.' || c == '_' {
                if c != '_' {
                    num_str.push(c);
                }
            } else {
                // Got a unit character
                let unit = if c == 'm' || c == 'u' || c == 'n' {
                    // Could be ms, us, ns, or just m (minutes) or s
                    let mut unit = String::from(c);
                    if let Some(&next) = chars.peek() {
                        if next == 's' {
                            unit.push(chars.next().unwrap());
                        }
                    }
                    unit
                } else {
                    c.to_string()
                };

                if num_str.is_empty() {
                    continue;
                }

                let value: f64 = num_str
                    .parse()
                    .map_err(|_| LiteralError::InvalidNumber(num_str.clone()))?;

                match unit.as_str() {
                    "d" => result.days = value as u32,
                    "h" => result.hours = value as u32,
                    "m" => result.minutes = value as u32,
                    "s" => {
                        result.seconds = value as u32;
                        // Handle fractional seconds
                        let frac = value - value.floor();
                        if frac > 0.0 {
                            result.milliseconds = (frac * 1000.0) as u32;
                        }
                    }
                    "ms" => result.milliseconds = value as u32,
                    "us" => result.microseconds = value as u32,
                    "ns" => result.nanoseconds = value as u32,
                    _ => return Err(LiteralError::InvalidUnit(unit)),
                }

                num_str.clear();
            }
        }

        Ok(result)
    }
}

impl fmt::Display for TimeLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T#")?;
        if self.negative {
            write!(f, "-")?;
        }
        let mut has_output = false;
        if self.days > 0 {
            write!(f, "{}d", self.days)?;
            has_output = true;
        }
        if self.hours > 0 {
            write!(f, "{}h", self.hours)?;
            has_output = true;
        }
        if self.minutes > 0 {
            write!(f, "{}m", self.minutes)?;
            has_output = true;
        }
        if self.seconds > 0 || self.milliseconds > 0 {
            if self.milliseconds > 0 {
                write!(f, "{}s{}ms", self.seconds, self.milliseconds)?;
            } else {
                write!(f, "{}s", self.seconds)?;
            }
            has_output = true;
        }
        if self.microseconds > 0 {
            write!(f, "{}us", self.microseconds)?;
            has_output = true;
        }
        if self.nanoseconds > 0 {
            write!(f, "{}ns", self.nanoseconds)?;
            has_output = true;
        }
        if !has_output {
            write!(f, "0s")?;
        }
        Ok(())
    }
}

/// Numeric literal with optional base
#[derive(Debug, Clone, PartialEq)]
pub enum NumericLiteral {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Real(f64),
}

impl NumericLiteral {
    /// Parse a numeric literal (supports 2#, 8#, 16# prefixes)
    pub fn parse(s: &str) -> Result<Self, LiteralError> {
        let s = s.trim().replace('_', "");

        // Check for base prefix
        if let Some(rest) = s.strip_prefix("16#") {
            let value = i64::from_str_radix(rest, 16)
                .map_err(|_| LiteralError::InvalidNumber(s.clone()))?;
            return Ok(NumericLiteral::Integer(value));
        }
        if let Some(rest) = s.strip_prefix("8#") {
            let value =
                i64::from_str_radix(rest, 8).map_err(|_| LiteralError::InvalidNumber(s.clone()))?;
            return Ok(NumericLiteral::Integer(value));
        }
        if let Some(rest) = s.strip_prefix("2#") {
            let value =
                i64::from_str_radix(rest, 2).map_err(|_| LiteralError::InvalidNumber(s.clone()))?;
            return Ok(NumericLiteral::Integer(value));
        }

        // Check for float
        if s.contains('.') || s.contains('e') || s.contains('E') {
            let value: f64 = s.parse().map_err(|_| LiteralError::InvalidNumber(s))?;
            return Ok(NumericLiteral::Real(value));
        }

        // Plain integer
        let value: i64 = s.parse().map_err(|_| LiteralError::InvalidNumber(s))?;
        Ok(NumericLiteral::Integer(value))
    }
}

/// Direct address (I/O addressing)
/// Examples: %IX0.0, %QW10, %MD100
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectAddress {
    /// Memory area: I (input), Q (output), M (memory)
    pub area: MemoryArea,
    /// Size: X (bit), B (byte), W (word), D (dword), L (lword)
    pub size: MemorySize,
    /// Address (byte offset)
    pub address: u32,
    /// Bit offset (for bit access)
    pub bit: Option<u8>,
}

/// Memory area for direct addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryArea {
    /// Input (%I)
    Input,
    /// Output (%Q)
    Output,
    /// Memory/Marker (%M)
    Memory,
}

/// Memory size for direct addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySize {
    /// Bit (X)
    Bit,
    /// Byte (B)
    Byte,
    /// Word (W) - 16 bits
    Word,
    /// Double word (D) - 32 bits
    DWord,
    /// Long word (L) - 64 bits
    LWord,
}

impl DirectAddress {
    /// Parse a direct address string (%IX0.0, %QW10, etc.)
    pub fn parse(s: &str) -> Result<Self, LiteralError> {
        let s = s.trim();

        if !s.starts_with('%') {
            return Err(LiteralError::InvalidPrefix("%"));
        }

        let s = &s[1..];
        let mut chars = s.chars();

        // Parse area
        let area = match chars.next() {
            Some('I') | Some('i') => MemoryArea::Input,
            Some('Q') | Some('q') => MemoryArea::Output,
            Some('M') | Some('m') => MemoryArea::Memory,
            _ => return Err(LiteralError::InvalidMemoryArea),
        };

        // Parse size (optional, default is bit for I/Q, byte for M)
        let next = chars.next().ok_or(LiteralError::InvalidAddress)?;
        let (size, first_digit) = match next {
            'X' | 'x' => (MemorySize::Bit, chars.next()),
            'B' | 'b' => (MemorySize::Byte, chars.next()),
            'W' | 'w' => (MemorySize::Word, chars.next()),
            'D' | 'd' => (MemorySize::DWord, chars.next()),
            'L' | 'l' => (MemorySize::LWord, chars.next()),
            c if c.is_ascii_digit() => (MemorySize::Bit, Some(c)),
            _ => return Err(LiteralError::InvalidMemorySize),
        };

        // Parse address
        let mut addr_str = String::new();
        if let Some(c) = first_digit {
            addr_str.push(c);
        }
        for c in chars {
            if c == '.' {
                break;
            }
            addr_str.push(c);
        }

        let address: u32 = addr_str
            .parse()
            .map_err(|_| LiteralError::InvalidAddress)?;

        // Parse bit offset if present
        let bit = if s.contains('.') {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() == 2 {
                Some(
                    parts[1]
                        .parse()
                        .map_err(|_| LiteralError::InvalidAddress)?,
                )
            } else {
                None
            }
        } else {
            None
        };

        Ok(DirectAddress {
            area,
            size,
            address,
            bit,
        })
    }
}

impl fmt::Display for DirectAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let area = match self.area {
            MemoryArea::Input => 'I',
            MemoryArea::Output => 'Q',
            MemoryArea::Memory => 'M',
        };
        let size = match self.size {
            MemorySize::Bit => 'X',
            MemorySize::Byte => 'B',
            MemorySize::Word => 'W',
            MemorySize::DWord => 'D',
            MemorySize::LWord => 'L',
        };

        if let Some(bit) = self.bit {
            write!(f, "%{}{}{}.{}", area, size, self.address, bit)
        } else {
            write!(f, "%{}{}{}", area, size, self.address)
        }
    }
}

/// Error type for literal parsing
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralError {
    /// Invalid prefix (expected T#, TIME#, etc.)
    InvalidPrefix(&'static str),
    /// Invalid number format
    InvalidNumber(String),
    /// Invalid unit (expected d, h, m, s, ms, etc.)
    InvalidUnit(String),
    /// Invalid memory area
    InvalidMemoryArea,
    /// Invalid memory size
    InvalidMemorySize,
    /// Invalid address format
    InvalidAddress,
}

impl fmt::Display for LiteralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralError::InvalidPrefix(expected) => {
                write!(f, "invalid prefix, expected {}", expected)
            }
            LiteralError::InvalidNumber(s) => write!(f, "invalid number: {}", s),
            LiteralError::InvalidUnit(s) => write!(f, "invalid unit: {}", s),
            LiteralError::InvalidMemoryArea => write!(f, "invalid memory area (expected I, Q, M)"),
            LiteralError::InvalidMemorySize => {
                write!(f, "invalid memory size (expected X, B, W, D, L)")
            }
            LiteralError::InvalidAddress => write!(f, "invalid address format"),
        }
    }
}

impl std::error::Error for LiteralError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_literal_parse() {
        let t = TimeLiteral::parse("T#1s").unwrap();
        assert_eq!(t.seconds, 1);
        assert_eq!(t.to_milliseconds(), 1000);

        let t = TimeLiteral::parse("T#1s500ms").unwrap();
        assert_eq!(t.seconds, 1);
        assert_eq!(t.milliseconds, 500);
        assert_eq!(t.to_milliseconds(), 1500);

        let t = TimeLiteral::parse("TIME#1d2h3m4s").unwrap();
        assert_eq!(t.days, 1);
        assert_eq!(t.hours, 2);
        assert_eq!(t.minutes, 3);
        assert_eq!(t.seconds, 4);

        let t = TimeLiteral::parse("T#-500ms").unwrap();
        assert!(t.negative);
        assert_eq!(t.to_milliseconds(), -500);
    }

    #[test]
    fn test_time_literal_display() {
        let t = TimeLiteral::from_milliseconds(1500);
        assert_eq!(t.to_string(), "T#1s500ms");
    }

    #[test]
    fn test_numeric_literal_parse() {
        assert_eq!(
            NumericLiteral::parse("100").unwrap(),
            NumericLiteral::Integer(100)
        );
        assert_eq!(
            NumericLiteral::parse("16#FF").unwrap(),
            NumericLiteral::Integer(255)
        );
        assert_eq!(
            NumericLiteral::parse("2#1010").unwrap(),
            NumericLiteral::Integer(10)
        );
        assert_eq!(
            NumericLiteral::parse("8#77").unwrap(),
            NumericLiteral::Integer(63)
        );
        assert_eq!(
            NumericLiteral::parse("3.14").unwrap(),
            NumericLiteral::Real(3.14)
        );
        assert_eq!(
            NumericLiteral::parse("1_000_000").unwrap(),
            NumericLiteral::Integer(1000000)
        );
    }

    #[test]
    fn test_direct_address_parse() {
        let addr = DirectAddress::parse("%IX0.0").unwrap();
        assert_eq!(addr.area, MemoryArea::Input);
        assert_eq!(addr.size, MemorySize::Bit);
        assert_eq!(addr.address, 0);
        assert_eq!(addr.bit, Some(0));

        let addr = DirectAddress::parse("%QW10").unwrap();
        assert_eq!(addr.area, MemoryArea::Output);
        assert_eq!(addr.size, MemorySize::Word);
        assert_eq!(addr.address, 10);
        assert_eq!(addr.bit, None);

        let addr = DirectAddress::parse("%MD100").unwrap();
        assert_eq!(addr.area, MemoryArea::Memory);
        assert_eq!(addr.size, MemorySize::DWord);
        assert_eq!(addr.address, 100);
    }

    #[test]
    fn test_direct_address_display() {
        let addr = DirectAddress {
            area: MemoryArea::Input,
            size: MemorySize::Bit,
            address: 0,
            bit: Some(5),
        };
        assert_eq!(addr.to_string(), "%IX0.5");
    }
}
