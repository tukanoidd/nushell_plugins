use nu_protocol::{FromValue, IntoValue};
use rgb::RGB8;
use smart_default::SmartDefault;
use textplots::LineStyle;

#[derive(Debug, Clone, FromValue, IntoValue)]
#[nu_value(type_name = "graph-config")]
pub struct GraphConfig {
    #[nu_value(rename = "type")]
    pub ty: Option<GraphType>,

    pub width: Option<u32>,
    pub height: Option<u32>,

    pub x_min: Option<f32>,
    pub x_max: Option<f32>,
    pub x_style: Option<GraphAxisStyle>,
    pub x_label: Option<String>,

    pub y_min: Option<f32>,
    pub y_max: Option<f32>,
    pub y_style: Option<GraphAxisStyle>,
    pub y_label: Option<String>,

    pub no_header: Option<bool>,
    pub no_axis: Option<bool>,
    pub no_borders: Option<bool>,
    pub no_figures: Option<bool>,

    pub lines: Vec<GraphLineConfig>,
}

#[derive(Debug, Clone, SmartDefault, FromValue, IntoValue)]
#[nu_value(type_name = "graph-type")]
pub enum GraphType {
    #[default]
    Points,
    Lines,
    Steps,
    Bars,
}

#[derive(Debug, SmartDefault, Clone, FromValue, IntoValue)]
#[nu_value(type_name = "graph-line-config")]
pub struct GraphLineConfig {
    pub color: Option<GraphLineColor>,
    pub points: Vec<GraphPoint>,
}

#[derive(Debug, SmartDefault, Clone, Copy, FromValue, IntoValue)]
#[nu_value(type_name = "graph-axis-style")]
pub enum GraphAxisStyle {
    None,
    #[default]
    Solid,
    Dotted,
    Dashed,
}

impl From<GraphAxisStyle> for LineStyle {
    fn from(value: GraphAxisStyle) -> Self {
        match value {
            GraphAxisStyle::None => Self::None,
            GraphAxisStyle::Solid => Self::Solid,
            GraphAxisStyle::Dotted => Self::Dotted,
            GraphAxisStyle::Dashed => Self::Dashed,
        }
    }
}

#[derive(Debug, Clone, Copy, FromValue, IntoValue)]
#[nu_value(type_name = "graph-line-color")]
pub struct GraphLineColor {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl From<GraphLineColor> for RGB8 {
    fn from(GraphLineColor { r, g, b }: GraphLineColor) -> Self {
        RGB8 {
            r: r.min(255) as u8,
            g: g.min(255) as u8,
            b: b.min(255) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy, FromValue, IntoValue)]
#[nu_value(type_name = "graph-point")]
pub struct GraphPoint {
    pub x: f32,
    pub y: f32,
}
