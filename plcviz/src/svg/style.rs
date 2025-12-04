//! SVG styling types

/// RGBA color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const GRAY: Color = Color::rgb(128, 128, 128);
    pub const LIGHT_GRAY: Color = Color::rgb(200, 200, 200);
    pub const BLUE: Color = Color::rgb(100, 149, 237);
    pub const GREEN: Color = Color::rgb(144, 238, 144);
    pub const YELLOW: Color = Color::rgb(255, 255, 224);
    pub const NONE: Color = Color::rgba(0, 0, 0, 0.0);

    pub fn to_svg(&self) -> String {
        if self.a == 0.0 {
            "none".to_string()
        } else if self.a < 1.0 {
            format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
        } else {
            format!("rgb({},{},{})", self.r, self.g, self.b)
        }
    }
}

/// Style for SVG elements
#[derive(Debug, Clone)]
pub struct Style {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f64,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: None,
            stroke_width: 1.0,
            font_family: None,
            font_size: None,
        }
    }
}

impl Style {
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn font(mut self, family: &str, size: f64) -> Self {
        self.font_family = Some(family.to_string());
        self.font_size = Some(size);
        self
    }

    pub fn to_svg_attrs(&self) -> String {
        let mut attrs = Vec::new();
        
        if let Some(ref fill) = self.fill {
            attrs.push(format!("fill=\"{}\"", fill.to_svg()));
        }
        
        if let Some(ref stroke) = self.stroke {
            attrs.push(format!("stroke=\"{}\"", stroke.to_svg()));
            attrs.push(format!("stroke-width=\"{}\"", self.stroke_width));
        }
        
        if let Some(ref font) = self.font_family {
            attrs.push(format!("font-family=\"{}\"", font));
        }
        
        if let Some(size) = self.font_size {
            attrs.push(format!("font-size=\"{}\"", size));
        }
        
        attrs.join(" ")
    }
}
