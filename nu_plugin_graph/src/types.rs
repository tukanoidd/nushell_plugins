use nu_protocol::{FromValue, IntoValue};
use rgb::RGB8;
use serde::{Deserialize, Serialize};
use textplots::LineStyle;

#[derive(Serialize, Deserialize, FromValue, IntoValue)]
pub enum GraphType {
    Points,
    Lines,
    Steps,
    Bars,
}

#[derive(Serialize, Deserialize, FromValue, IntoValue)]
pub struct GraphPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, FromValue, IntoValue)]
pub enum GraphLineStyle {
    None,
    Solid,
    Dotted,
    Dashed,
}

impl From<GraphLineStyle> for LineStyle {
    fn from(value: GraphLineStyle) -> Self {
        match value {
            GraphLineStyle::None => Self::None,
            GraphLineStyle::Solid => Self::Solid,
            GraphLineStyle::Dotted => Self::Dotted,
            GraphLineStyle::Dashed => Self::Dashed,
        }
    }
}

#[derive(Serialize, Deserialize, FromValue, IntoValue)]
pub struct GraphColor {
    pub r: u16,
    pub g: u16,
    pub b: u16,
}

impl From<GraphColor> for RGB8 {
    fn from(GraphColor { r, g, b }: GraphColor) -> Self {
        RGB8 {
            r: r.min(255) as u8,
            g: g.min(255) as u8,
            b: b.min(255) as u8,
        }
    }
}
