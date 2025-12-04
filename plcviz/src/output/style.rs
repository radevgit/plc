//! SVG style definitions

use std::fmt;

/// Color representation
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    /// Hex color like "#1a5f7a"
    Hex(String),
    /// RGB color
    Rgb(u8, u8, u8),
    /// RGBA with alpha
    Rgba(u8, u8, u8, f32),
    /// Named color like "white", "black"
    Named(String),
    /// No color (transparent)
    None,
}

impl Color {
    pub fn hex(s: &str) -> Self {
        Color::Hex(s.to_string())
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb(r, g, b)
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Color::Rgba(r, g, b, a)
    }

    pub fn named(s: &str) -> Self {
        Color::Named(s.to_string())
    }

    // Predefined colors
    pub fn white() -> Self { Color::Named("white".into()) }
    pub fn black() -> Self { Color::Named("black".into()) }
    pub fn none() -> Self { Color::None }

    // PLCeye theme colors
    pub fn primary() -> Self { Color::hex("#1a5f7a") }
    pub fn secondary() -> Self { Color::hex("#0d2d3a") }
    pub fn accent() -> Self { Color::hex("#2ecc71") }
    pub fn warning() -> Self { Color::hex("#f39c12") }
    pub fn error() -> Self { Color::hex("#e74c3c") }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Hex(s) => write!(f, "{}", s),
            Color::Rgb(r, g, b) => write!(f, "rgb({},{},{})", r, g, b),
            Color::Rgba(r, g, b, a) => write!(f, "rgba({},{},{},{})", r, g, b, a),
            Color::Named(s) => write!(f, "{}", s),
            Color::None => write!(f, "none"),
        }
    }
}

/// Style attributes for SVG elements
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: Option<f64>,
    pub opacity: Option<f64>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_weight: Option<String>,
    pub text_anchor: Option<String>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn font_family(mut self, family: &str) -> Self {
        self.font_family = Some(family.to_string());
        self
    }

    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn font_weight(mut self, weight: &str) -> Self {
        self.font_weight = Some(weight.to_string());
        self
    }

    pub fn text_anchor(mut self, anchor: &str) -> Self {
        self.text_anchor = Some(anchor.to_string());
        self
    }

    /// Convert style to SVG attribute string
    pub fn to_attrs(&self) -> String {
        let mut attrs = Vec::new();

        if let Some(ref fill) = self.fill {
            attrs.push(format!(r#"fill="{}""#, fill));
        }
        if let Some(ref stroke) = self.stroke {
            attrs.push(format!(r#"stroke="{}""#, stroke));
        }
        if let Some(width) = self.stroke_width {
            attrs.push(format!(r#"stroke-width="{}""#, width));
        }
        if let Some(opacity) = self.opacity {
            attrs.push(format!(r#"opacity="{}""#, opacity));
        }
        if let Some(ref family) = self.font_family {
            attrs.push(format!(r#"font-family="{}""#, family));
        }
        if let Some(size) = self.font_size {
            attrs.push(format!(r#"font-size="{}""#, size));
        }
        if let Some(ref weight) = self.font_weight {
            attrs.push(format!(r#"font-weight="{}""#, weight));
        }
        if let Some(ref anchor) = self.text_anchor {
            attrs.push(format!(r#"text-anchor="{}""#, anchor));
        }

        attrs.join(" ")
    }
}

// Predefined styles
impl Style {
    /// Style for graph nodes
    pub fn node() -> Self {
        Self::new()
            .fill(Color::primary())
            .stroke(Color::secondary())
            .stroke_width(1.5)
    }

    /// Style for node labels
    pub fn node_label() -> Self {
        Self::new()
            .fill(Color::white())
            .font_family("sans-serif")
            .font_size(12.0)
            .text_anchor("middle")
    }

    /// Style for edges/arrows
    pub fn edge() -> Self {
        Self::new()
            .stroke(Color::hex("#666"))
            .stroke_width(1.5)
            .fill(Color::none())
    }

    /// Style for highlighted elements
    pub fn highlighted() -> Self {
        Self::new()
            .stroke(Color::error())
            .stroke_width(2.5)
    }
}
