//! SVG document builder
//!
//! Builds complete SVG documents from elements.

use super::elements::*;
use super::style::Color;

/// SVG document builder
pub struct SvgBuilder {
    width: u32,
    height: u32,
    view_box: Option<(f64, f64, f64, f64)>,
    background: Option<Color>,
    defs: Vec<String>,
    elements: Vec<String>,
    styles: Vec<String>,
}

impl SvgBuilder {
    /// Create new SVG builder with dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            view_box: None,
            background: None,
            defs: Vec::new(),
            elements: Vec::new(),
            styles: Vec::new(),
        }
    }

    /// Set viewBox (for scaling)
    pub fn view_box(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.view_box = Some((x, y, width, height));
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Add a definition (markers, gradients, etc.)
    pub fn add_def(&mut self, def: String) {
        self.defs.push(def);
    }

    /// Add arrow marker to defs
    pub fn with_arrow_marker(mut self, id: &str, color: &str) -> Self {
        self.defs.push(arrow_marker(id, color));
        self
    }

    /// Add default arrow marker
    pub fn with_default_arrows(mut self) -> Self {
        self.defs.push(arrow_marker("arrow", "#666"));
        self.defs.push(arrow_marker("arrow-highlight", "#e74c3c"));
        self
    }

    /// Add CSS style
    pub fn add_style(&mut self, css: &str) {
        self.styles.push(css.to_string());
    }

    /// Add default styles for graphs
    pub fn with_default_styles(mut self) -> Self {
        self.styles.push(r#"
            .node rect { filter: drop-shadow(2px 2px 3px rgba(0,0,0,0.2)); }
            .node text { pointer-events: none; }
            .edge { transition: stroke 0.2s; }
            .edge:hover { stroke: #e74c3c; stroke-width: 2.5; }
        "#.to_string());
        self
    }

    /// Add an element
    pub fn add(&mut self, element: String) {
        self.elements.push(element);
    }

    /// Add multiple elements
    pub fn add_all(&mut self, elements: &[String]) {
        self.elements.extend(elements.iter().cloned());
    }

    /// Build the final SVG string
    pub fn build(self) -> String {
        let mut svg = String::new();

        // Opening tag
        let view_box_attr = self.view_box
            .map(|(x, y, w, h)| format!(r#" viewBox="{} {} {} {}""#, x, y, w, h))
            .unwrap_or_default();

        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}"{}>
"#,
            self.width, self.height, view_box_attr
        ));

        // Styles
        if !self.styles.is_empty() {
            svg.push_str("<style>\n");
            for style in &self.styles {
                svg.push_str(style);
                svg.push('\n');
            }
            svg.push_str("</style>\n");
        }

        // Defs
        if !self.defs.is_empty() {
            svg.push_str("<defs>\n");
            for def in &self.defs {
                svg.push_str(def);
                svg.push('\n');
            }
            svg.push_str("</defs>\n");
        }

        // Background
        if let Some(color) = self.background {
            svg.push_str(&format!(
                r#"<rect width="100%" height="100%" fill="{}"/>
"#,
                color
            ));
        }

        // Elements
        for element in &self.elements {
            svg.push_str(element);
            svg.push('\n');
        }

        // Closing tag
        svg.push_str("</svg>");

        svg
    }
}

impl Default for SvgBuilder {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_svg() {
        let svg = SvgBuilder::new(400, 300).build();
        assert!(svg.contains(r#"width="400""#));
        assert!(svg.contains(r#"height="300""#));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_with_elements() {
        let mut builder = SvgBuilder::new(400, 300);
        builder.add(node_box(50.0, 50.0, 100.0, 40.0, "Test"));
        let svg = builder.build();
        assert!(svg.contains("Test"));
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_with_defs() {
        let svg = SvgBuilder::new(400, 300)
            .with_default_arrows()
            .build();
        assert!(svg.contains("<defs>"));
        assert!(svg.contains(r#"id="arrow""#));
    }
}
