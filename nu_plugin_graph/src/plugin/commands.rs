use nu_plugin::PluginCommand;
use nu_protocol::{FromValue, Signature, SyntaxShape, Type, Value};
use textplots::{AxisBuilder, Chart, ColorPlot, Plot, Shape};

use crate::types::{GraphColor, GraphLineStyle, GraphPoint, GraphType};

use super::NuGraphsPlugin;

pub struct Draw;

impl PluginCommand for Draw {
    type Plugin = NuGraphsPlugin;

    fn name(&self) -> &str {
        "draw"
    }

    fn signature(&self) -> Signature {
        Signature::build("draw")
            // Input
            .input_output_type(
                Type::Table(Box::new([
                    ("x".into(), Type::Float),
                    ("y".into(), Type::Float),
                ])),
                Type::String,
            )
            // General
            .named("type", SyntaxShape::String, "Type of graph", Some('t'))
            .named(
                "line-color",
                SyntaxShape::Record(vec![
                    ("r".into(), SyntaxShape::Int),
                    ("g".into(), SyntaxShape::Int),
                    ("b".into(), SyntaxShape::Int),
                ]),
                "Color of the line",
                Some('c'),
            )
            .named("width", SyntaxShape::Int, "Width of the chart", Some('W'))
            .named("height", SyntaxShape::Int, "Height of the chart", Some('H'))
            // X Axis
            .named("x-min", SyntaxShape::Float, "Minimum X Value", None)
            .named("x-max", SyntaxShape::Float, "Maximum X Value", None)
            .named(
                "x-axis-style",
                SyntaxShape::String,
                "Style of the X Axis line",
                None,
            )
            .named(
                "x-axis-label",
                SyntaxShape::String,
                "Label for the X Axis",
                Some('x'),
            )
            // Y Axis
            .named("y-min", SyntaxShape::Float, "Minimum Y Value", None)
            .named("y-max", SyntaxShape::Float, "Maximum Y Value", None)
            .named(
                "y-axis-style",
                SyntaxShape::String,
                "Style of the Y Axis line",
                None,
            )
            .named(
                "y-axis-label",
                SyntaxShape::String,
                "Label for the Y Axis",
                Some('y'),
            )
            // Remove From Graph
            .named(
                "no-header",
                SyntaxShape::Boolean,
                "Don't show the header",
                None,
            )
            .named(
                "no-borders",
                SyntaxShape::Boolean,
                "Don't draw borders",
                Some('B'),
            )
            .named(
                "no-axis",
                SyntaxShape::Boolean,
                "Don't draw axis",
                Some('A'),
            )
            .named(
                "no-figures",
                SyntaxShape::Boolean,
                "Don't draw figures",
                Some('F'),
            )
    }

    fn description(&self) -> &str {
        "Draw a chart"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &nu_plugin::EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        let input_span = input.span().unwrap_or(call.head);

        let mut input = Vec::<GraphPoint>::from_value(input.into_value(input_span)?)?;
        input.sort_by_key(|p| (p.x * (p.x.fract().to_string().len() - 2) as f32) as i64);

        macro_rules! extract_flag {
            ($name:literal | $ty:ty $(| $default:expr)?) => {
                call.get_flag::<$ty>($name)?$(.unwrap_or_else(|| $default))?
            };
        }

        let ty = extract_flag!("type" | GraphType | GraphType::Points);

        let line_color = extract_flag!("line-color" | GraphColor);

        let width = extract_flag!("width" | u32 | 120);
        let height = extract_flag!("height" | u32 | 60);

        let x_min = extract_flag!("x-min" | f32 | input.first().map(|p| p.x).unwrap_or(f32::MIN));
        let x_max = extract_flag!("x-max" | f32 | input.last().map(|p| p.x).unwrap_or(f32::MAX));
        let x_axis_style = extract_flag!("x-axis-style" | GraphLineStyle | GraphLineStyle::Solid);
        let x_axis_label = extract_flag!("x-axis-label" | String | "X".into());

        let y_min = extract_flag!("y-min" | f32);
        let y_max = extract_flag!("y-may" | f32);
        let y_axis_style = extract_flag!("y-axis-style" | GraphLineStyle | GraphLineStyle::Solid);
        let y_axis_label = extract_flag!("y-axis-label" | String | "Y".into());

        let no_header = extract_flag!("no-header" | bool | false);
        let no_borders = extract_flag!("no-borders" | bool | false);
        let no_axis = extract_flag!("no-axis" | bool | false);
        let no_figures = extract_flag!("no-figures" | bool | false);

        let mut chart = match (y_min, y_max) {
            (Some(y_min), Some(y_max)) => {
                Chart::new_with_y_range(width, height, x_min, x_max, y_min, y_max)
            }
            _ => Chart::new(width, height, x_min, x_max),
        };

        let inputs_shape_list = input
            .iter()
            .map(|GraphPoint { x, y }| (*x, *y))
            .collect::<Vec<_>>();
        let input_shape = match ty {
            GraphType::Points => Shape::Points(&inputs_shape_list),
            GraphType::Lines => Shape::Lines(&inputs_shape_list),
            GraphType::Steps => Shape::Steps(&inputs_shape_list),
            GraphType::Bars => Shape::Bars(&inputs_shape_list),
        };

        let get_chart_str = |chart: &mut Chart<'_>| {
            if !no_borders {
                chart.borders();
            }

            if !no_axis {
                chart.axis();
            }

            if !no_figures {
                chart.figures();
            }

            chart.to_string()
        };
        let chart_str = get_chart_str(match line_color {
            Some(color) => chart
                .x_axis_style(x_axis_style.into())
                .y_axis_style(y_axis_style.into())
                .linecolorplot(&input_shape, color.into()),
            None => chart
                .x_axis_style(x_axis_style.into())
                .y_axis_style(y_axis_style.into())
                .lineplot(&input_shape),
        });

        let resulting_str = (match no_header {
            true => vec![chart_str],
            false => vec![format!("{y_axis_label}/{x_axis_label}"), chart_str],
        })
        .join("\n");

        Ok(nu_protocol::PipelineData::Value(
            Value::string(resulting_str, call.head),
            None,
        ))
    }
}
