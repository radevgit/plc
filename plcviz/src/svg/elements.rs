//! SVG elements

use super::style::Style;

/// Point in 2D space
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A rectangle element
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rx: Option<f64>,
    pub style: Style,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            rx: None,
            style: Style::default(),
        }
    }

    pub fn rounded(mut self, radius: f64) -> Self {
        self.rx = Some(radius);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn to_svg(&self) -> String {
        let mut attrs = format!(
            "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
            self.x, self.y, self.width, self.height
        );
        if let Some(rx) = self.rx {
            attrs.push_str(&format!(" rx=\"{}\"", rx));
        }
        let style_attrs = self.style.to_svg_attrs();
        if !style_attrs.is_empty() {
            attrs.push_str(&format!(" {}", style_attrs));
        }
        format!("<rect {}/>", attrs)
    }
}

/// A text element
#[derive(Debug, Clone)]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub content: String,
    pub anchor: TextAnchor,
    pub style: Style,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

impl Text {
    pub fn new(x: f64, y: f64, content: &str) -> Self {
        Self {
            x,
            y,
            content: content.to_string(),
            anchor: TextAnchor::Start,
            style: Style::default(),
        }
    }

    pub fn anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn to_svg(&self) -> String {
        let anchor_str = match self.anchor {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        };
        let style_attrs = self.style.to_svg_attrs();
        let style_part = if style_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", style_attrs)
        };
        format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\"{}>{}</text>",
            self.x, self.y, anchor_str, style_part,
            xml_escape(&self.content)
        )
    }
}

/// A line element
#[derive(Debug, Clone)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub style: Style,
}

impl Line {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn to_svg(&self) -> String {
        let style_attrs = self.style.to_svg_attrs();
        let style_part = if style_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", style_attrs)
        };
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\"{}/>",
            self.x1, self.y1, self.x2, self.y2, style_part
        )
    }
}

/// A circle element
#[derive(Debug, Clone)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub style: Style,
}

impl Circle {
    pub fn new(cx: f64, cy: f64, r: f64) -> Self {
        Self {
            cx,
            cy,
            r,
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn to_svg(&self) -> String {
        let style_attrs = self.style.to_svg_attrs();
        let style_part = if style_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", style_attrs)
        };
        format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"{}/>",
            self.cx, self.cy, self.r, style_part
        )
    }
}

/// A path element with SVG path data
#[derive(Debug, Clone)]
pub struct Path {
    pub d: String,
    pub style: Style,
    pub marker_end: Option<String>,
}

impl Path {
    pub fn new(d: &str) -> Self {
        Self {
            d: d.to_string(),
            style: Style::default(),
            marker_end: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn marker_end(mut self, marker_id: &str) -> Self {
        self.marker_end = Some(marker_id.to_string());
        self
    }

    pub fn to_svg(&self) -> String {
        let style_attrs = self.style.to_svg_attrs();
        let style_part = if style_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", style_attrs)
        };
        let marker_part = match &self.marker_end {
            Some(id) => format!(" marker-end=\"url(#{})\"", id),
            None => String::new(),
        };
        format!("<path d=\"{}\"{}{}/>", self.d, style_part, marker_part)
    }
}

/// A polyline (connected line segments)
#[derive(Debug, Clone)]
pub struct Polyline {
    pub points: Vec<Point>,
    pub style: Style,
    pub marker_end: Option<String>,
}

impl Polyline {
    pub fn new(points: Vec<Point>) -> Self {
        Self {
            points,
            style: Style::default(),
            marker_end: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn marker_end(mut self, marker_id: &str) -> Self {
        self.marker_end = Some(marker_id.to_string());
        self
    }

    pub fn to_svg(&self) -> String {
        let points_str: String = self
            .points
            .iter()
            .map(|p| format!("{},{}", p.x, p.y))
            .collect::<Vec<_>>()
            .join(" ");
        let style_attrs = self.style.to_svg_attrs();
        let style_part = if style_attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", style_attrs)
        };
        let marker_part = match &self.marker_end {
            Some(id) => format!(" marker-end=\"url(#{})\"", id),
            None => String::new(),
        };
        format!("<polyline points=\"{}\"{}{}/>", points_str, style_part, marker_part)
    }
}

/// A group element
#[derive(Debug, Clone)]
pub struct Group {
    pub id: Option<String>,
    pub class: Option<String>,
    pub elements: Vec<String>,
}

impl Group {
    pub fn new() -> Self {
        Self {
            id: None,
            class: None,
            elements: Vec::new(),
        }
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    pub fn add(&mut self, element: String) {
        self.elements.push(element);
    }

    pub fn to_svg(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(ref id) = self.id {
            attrs.push(format!("id=\"{}\"", id));
        }
        if let Some(ref class) = self.class {
            attrs.push(format!("class=\"{}\"", class));
        }
        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };
        let content = self.elements.join("\n");
        format!("<g{}>\n{}\n</g>", attrs_str, content)
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special XML characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
