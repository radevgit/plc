//! Custom SVG renderer implementing layout-rs RenderBackend

use layout::core::format::{ClipHandle, RenderBackend};
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;

/// Custom SVG renderer that produces clean, valid SVG
pub struct SvgRenderer {
    content: String,
    width: f64,
    height: f64,
}

impl SvgRenderer {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    fn grow(&mut self, x: f64, y: f64) {
        self.width = self.width.max(x + 20.0);
        self.height = self.height.max(y + 20.0);
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    /// Convert layout-rs color to standard SVG color (6-digit hex)
    fn color_to_svg(color: &layout::core::color::Color) -> String {
        let web = color.to_web_color();
        // layout-rs returns #rrggbbaa, we want #rrggbb
        if web.len() == 9 && web.starts_with('#') {
            format!("#{}", &web[1..7])
        } else {
            web
        }
    }

    /// Convert optional fill color to SVG
    fn fill_to_svg(fill: Option<layout::core::color::Color>) -> String {
        fill.map(|c| Self::color_to_svg(&c))
            .unwrap_or_else(|| "#fff".to_string())
    }

    /// Generate the final SVG document
    pub fn finalize(&self) -> String {
        let mut svg = String::new();

        // Standard SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
            self.width as u32,
            self.height as u32,
            self.width as u32,
            self.height as u32
        ));

        // Defs for arrow markers
        // refX=10 positions arrow tip at exact endpoint, but we want gap
        // markerWidth=8, markerHeight=6 for smaller arrows
        svg.push_str(
            r##"<defs>
  <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="#333"/>
  </marker>
</defs>
"##,
        );

        // Background
        svg.push_str(r##"<rect width="100%" height="100%" fill="#fafafa"/>
"##);

        // Content
        svg.push_str(&self.content);

        svg.push_str("</svg>\n");
        svg
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for SvgRenderer {
    fn draw_rect(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        _properties: Option<String>,
        _clip: Option<ClipHandle>,
    ) {
        self.grow(xy.x + size.x, xy.y + size.y);

        let fill = Self::fill_to_svg(look.fill_color);
        let stroke = Self::color_to_svg(&look.line_color);
        let rx = if look.rounded > 0 { look.rounded } else { 4 };

        self.content.push_str(&format!(
            r#"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="{}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
            xy.x, xy.y, size.x, size.y, rx, fill, stroke, look.line_width
        ));
    }

    fn draw_line(
        &mut self,
        start: Point,
        stop: Point,
        look: &StyleAttr,
        _properties: Option<String>,
    ) {
        self.grow(start.x.max(stop.x), start.y.max(stop.y));
        let stroke = Self::color_to_svg(&look.line_color);

        self.content.push_str(&format!(
            r#"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{}" stroke-width="{}"/>
"#,
            start.x, start.y, stop.x, stop.y, stroke, look.line_width
        ));
    }

    fn draw_circle(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        _properties: Option<String>,
    ) {
        self.grow(xy.x + size.x, xy.y + size.y);

        let fill = Self::fill_to_svg(look.fill_color);
        let stroke = Self::color_to_svg(&look.line_color);

        self.content.push_str(&format!(
            r#"<ellipse cx="{:.1}" cy="{:.1}" rx="{:.1}" ry="{:.1}" fill="{}" stroke="{}" stroke-width="{}"/>
"#,
            xy.x,
            xy.y,
            size.x / 2.0,
            size.y / 2.0,
            fill,
            stroke,
            look.line_width
        ));
    }

    fn draw_text(&mut self, xy: Point, text: &str, look: &StyleAttr) {
        self.grow(xy.x + 50.0, xy.y + 20.0);

        self.content.push_str(&format!(
            r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" dominant-baseline="middle" font-family="sans-serif" font-size="{}">{}</text>
"#,
            xy.x,
            xy.y,
            look.font_size,
            Self::escape_xml(text)
        ));
    }

    fn draw_arrow(
        &mut self,
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        _properties: Option<String>,
        _text: &str,
    ) {
        if path.is_empty() {
            return;
        }

        for (p1, p2) in path {
            self.grow(p1.x.max(p2.x), p1.y.max(p2.y));
        }

        let stroke = Self::color_to_svg(&look.line_color);
        let dash = if dashed {
            r#" stroke-dasharray="5,3""#
        } else {
            ""
        };
        let marker = if head.1 {
            r#" marker-end="url(#arrowhead)""#
        } else {
            ""
        };

        // Build path data - cubic bezier curves
        let mut d = String::new();

        // First segment: M start C control1, control2, end
        d.push_str(&format!("M {:.1},{:.1}", path[0].0.x, path[0].0.y));

        if path.len() >= 2 {
            d.push_str(&format!(
                " C {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                path[0].1.x, path[0].1.y, path[1].0.x, path[1].0.y, path[1].1.x, path[1].1.y
            ));
        }

        // Additional segments use S (smooth curve)
        for point in path.iter().skip(2) {
            d.push_str(&format!(
                " S {:.1},{:.1} {:.1},{:.1}",
                point.0.x, point.0.y, point.1.x, point.1.y
            ));
        }

        self.content.push_str(&format!(
            r#"<path d="{}" fill="none" stroke="{}"{}{} stroke-width="{}"/>
"#,
            d, stroke, dash, marker, look.line_width
        ));
    }

    fn create_clip(&mut self, _xy: Point, _size: Point, _rounded_px: usize) -> ClipHandle {
        0 // We don't use clipping for now
    }
}
