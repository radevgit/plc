//! SVG document builder

use super::style::Color;

/// Builder for constructing complete SVG documents
pub struct SvgBuilder {
    width: f64,
    height: f64,
    viewbox: Option<(f64, f64, f64, f64)>,
    elements: Vec<String>,
    defs: Vec<String>,
}

impl SvgBuilder {
    pub fn new(width: f64, height: f64) -> Self {
        let mut builder = Self {
            width,
            height,
            viewbox: None,
            elements: Vec::new(),
            defs: Vec::new(),
        };
        // Add default arrow marker
        builder.add_arrow_marker("arrow", Color::BLACK);
        builder
    }

    pub fn viewbox(mut self, x: f64, y: f64, w: f64, h: f64) -> Self {
        self.viewbox = Some((x, y, w, h));
        self
    }

    /// Add an arrow marker definition
    pub fn add_arrow_marker(&mut self, id: &str, color: Color) {
        let marker = format!(
            r#"<marker id="{}" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
  <path d="M 0 0 L 10 5 L 0 10 z" fill="{}"/>
</marker>"#,
            id,
            color.to_svg()
        );
        self.defs.push(marker);
    }

    /// Add raw SVG content
    pub fn add(&mut self, svg: String) {
        self.elements.push(svg);
    }

    /// Add a definition to defs section
    pub fn add_def(&mut self, def: String) {
        self.defs.push(def);
    }

    /// Build the complete SVG document
    pub fn build(&self) -> String {
        let viewbox_attr = match self.viewbox {
            Some((x, y, w, h)) => format!(" viewBox=\"{} {} {} {}\"", x, y, w, h),
            None => format!(" viewBox=\"0 0 {} {}\"", self.width, self.height),
        };

        let defs_section = if self.defs.is_empty() {
            String::new()
        } else {
            format!("<defs>\n{}\n</defs>\n", self.defs.join("\n"))
        };

        let content = self.elements.join("\n");

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}"{}>
{}{}
</svg>"#,
            self.width, self.height, viewbox_attr, defs_section, content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg::{Rect, Style, Text, TextAnchor};

    #[test]
    fn test_simple_svg() {
        let mut builder = SvgBuilder::new(200.0, 100.0);
        
        let rect = Rect::new(10.0, 10.0, 80.0, 40.0)
            .style(Style::default().fill(Color::LIGHT_GRAY).stroke(Color::BLACK));
        builder.add(rect.to_svg());
        
        let text = Text::new(50.0, 35.0, "Hello")
            .anchor(TextAnchor::Middle)
            .style(Style::default().font("Arial", 14.0));
        builder.add(text.to_svg());
        
        let svg = builder.build();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<text"));
        assert!(svg.contains("Hello"));
    }

    #[test]
    fn test_arrow_marker() {
        let builder = SvgBuilder::new(100.0, 100.0);
        let svg = builder.build();
        assert!(svg.contains("<marker id=\"arrow\""));
        assert!(svg.contains("<defs>"));
    }
}
