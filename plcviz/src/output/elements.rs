//! SVG element functions
//!
//! Each function generates an SVG element as a String.
//! Composable and customizable.

use super::style::Style;

/// Rectangle element
pub fn rect(x: f64, y: f64, width: f64, height: f64, style: &Style) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" {}/>"#,
        x, y, width, height, style.to_attrs()
    )
}

/// Rounded rectangle element
pub fn rect_rounded(x: f64, y: f64, width: f64, height: f64, rx: f64, style: &Style) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" {}/>"#,
        x, y, width, height, rx, style.to_attrs()
    )
}

/// Circle element
pub fn circle(cx: f64, cy: f64, r: f64, style: &Style) -> String {
    format!(
        r#"<circle cx="{}" cy="{}" r="{}" {}/>"#,
        cx, cy, r, style.to_attrs()
    )
}

/// Ellipse element
pub fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64, style: &Style) -> String {
    format!(
        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" {}/>"#,
        cx, cy, rx, ry, style.to_attrs()
    )
}

/// Line element
pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, style: &Style) -> String {
    format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {}/>"#,
        x1, y1, x2, y2, style.to_attrs()
    )
}

/// Polyline element (multiple connected lines)
pub fn polyline(points: &[(f64, f64)], style: &Style) -> String {
    let points_str: String = points
        .iter()
        .map(|(x, y)| format!("{},{}", x, y))
        .collect::<Vec<_>>()
        .join(" ");
    
    format!(
        r#"<polyline points="{}" {}/>"#,
        points_str, style.to_attrs()
    )
}

/// Path element with raw path data
pub fn path(d: &str, style: &Style) -> String {
    format!(
        r#"<path d="{}" {}/>"#,
        d, style.to_attrs()
    )
}

/// Bezier curve path (cubic)
pub fn bezier(x1: f64, y1: f64, cx1: f64, cy1: f64, cx2: f64, cy2: f64, x2: f64, y2: f64, style: &Style) -> String {
    let d = format!("M{},{} C{},{} {},{} {},{}", x1, y1, cx1, cy1, cx2, cy2, x2, y2);
    path(&d, style)
}

/// Quadratic bezier curve
pub fn quad_bezier(x1: f64, y1: f64, cx: f64, cy: f64, x2: f64, y2: f64, style: &Style) -> String {
    let d = format!("M{},{} Q{},{} {},{}", x1, y1, cx, cy, x2, y2);
    path(&d, style)
}

/// Text element
pub fn text(x: f64, y: f64, content: &str, style: &Style) -> String {
    format!(
        r#"<text x="{}" y="{}" {}>{}</text>"#,
        x, y, style.to_attrs(), escape_xml(content)
    )
}

/// Text with explicit dy offset (useful for vertical centering)
pub fn text_dy(x: f64, y: f64, dy: f64, content: &str, style: &Style) -> String {
    format!(
        r#"<text x="{}" y="{}" dy="{}" {}>{}</text>"#,
        x, y, dy, style.to_attrs(), escape_xml(content)
    )
}

/// Group element - wraps children
pub fn group(children: &[String]) -> String {
    format!("<g>\n{}\n</g>", children.join("\n"))
}

/// Group with transform
pub fn group_transform(transform: &str, children: &[String]) -> String {
    format!(
        r#"<g transform="{}">{}</g>"#,
        transform,
        children.join("\n")
    )
}

/// Group with translation
pub fn group_translate(x: f64, y: f64, children: &[String]) -> String {
    group_transform(&format!("translate({},{})", x, y), children)
}

/// Group with id and class
pub fn group_id_class(id: &str, class: &str, children: &[String]) -> String {
    format!(
        r#"<g id="{}" class="{}">{}</g>"#,
        id, class,
        children.join("\n")
    )
}

/// Arrow marker definition (for use in defs)
pub fn arrow_marker(id: &str, color: &str) -> String {
    format!(
        r#"<marker id="{}" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
  <path d="M 0 0 L 10 5 L 0 10 z" fill="{}"/>
</marker>"#,
        id, color
    )
}

/// Title element (for tooltips)
pub fn title(content: &str) -> String {
    format!("<title>{}</title>", escape_xml(content))
}

/// Hyperlink wrapper
pub fn link(href: &str, children: &[String]) -> String {
    format!(
        r#"<a href="{}">{}</a>"#,
        href,
        children.join("\n")
    )
}

// === Composite elements (higher-level) ===

/// Node box with label (common graph node)
pub fn node_box(x: f64, y: f64, width: f64, height: f64, label: &str) -> String {
    let box_style = Style::node();
    let label_style = Style::node_label();
    
    let rect = rect_rounded(x, y, width, height, 5.0, &box_style);
    let text = text_dy(x + width / 2.0, y + height / 2.0, 4.0, label, &label_style);
    
    group(&[rect, text])
}

/// Arrow edge between two points
pub fn arrow_edge(x1: f64, y1: f64, x2: f64, y2: f64, marker_id: &str) -> String {
    let style = Style::edge();
    format!(
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {} marker-end="url(#{})"/>"#,
        x1, y1, x2, y2, style.to_attrs(), marker_id
    )
}

/// Curved arrow edge (bezier)
pub fn arrow_edge_curved(x1: f64, y1: f64, x2: f64, y2: f64, marker_id: &str) -> String {
    let style = Style::edge();
    // Control points for a nice curve
    let cy1 = y1 + (y2 - y1) * 0.3;
    let cy2 = y2 - (y2 - y1) * 0.3;
    
    format!(
        r#"<path d="M{},{} C{},{} {},{} {},{}" {} marker-end="url(#{})"/>"#,
        x1, y1, x1, cy1, x2, cy2, x2, y2,
        style.to_attrs(), marker_id
    )
}

// === Helper functions ===

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let style = Style::new().fill(super::super::style::Color::hex("#fff"));
        let svg = rect(10.0, 20.0, 100.0, 50.0, &style);
        assert!(svg.contains("x=\"10\""));
        assert!(svg.contains("width=\"100\""));
        assert!(svg.contains("fill=\"#fff\""));
    }

    #[test]
    fn test_text_escaping() {
        let style = Style::new();
        let svg = text(0.0, 0.0, "<Test & \"Value\">", &style);
        assert!(svg.contains("&lt;Test &amp; &quot;Value&quot;&gt;"));
    }

    #[test]
    fn test_node_box() {
        let svg = node_box(100.0, 100.0, 120.0, 40.0, "MainRoutine");
        assert!(svg.contains("<g>"));
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<text"));
        assert!(svg.contains("MainRoutine"));
    }
}
