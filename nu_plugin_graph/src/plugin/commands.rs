use nu_plugin::PluginCommand;
use nu_protocol::{Signature, SyntaxShape, Type, Value};
use textplots::{AxisBuilder, Chart, ColorPlot, Plot, Shape};

use crate::types::{GraphConfig, GraphLineConfig, GraphPoint, GraphType};

use super::NuGraphsPlugin;

pub struct Draw;

impl PluginCommand for Draw {
    type Plugin = NuGraphsPlugin;

    fn name(&self) -> &str {
        "draw"
    }

    fn signature(&self) -> Signature {
        Signature::build("draw").input_output_type(Type::Nothing, Type::String).required(
            "graph-config",
            SyntaxShape::Any,
            [
                "\n\tGraph Configuration {",
                &[
                    "\ttype: Type of Graph ([points*, lines, steps, bars])",
                    "width: Width of Graph (default: 120)",
                    "width: Height of Graph (default: 60)",
                    "x_min: Minimum Value Of X (default: Minimum value of x across the lines)",
                    "x_min: Maximum Value Of X (default: Maximum value of x across the lines)",
                    "x_style: Style of the Graphs X Axis ([none, solid*, dotted, dashed])",
                    "x_label: Label of the X Axis (default: 'X')",
                    "y_min: Minimum Value Of Y (default: Minimum value of y across the lines)",
                    "y_min: Maximum Value Of Y (default: Maximum value of y across the lines)",
                    "y_style: Style of the Graphs Y Axis ([none, solid*, dotted, dashed])",
                    "y_label: Label of the Y Axis (default: 'Y')",
                    "no_header: Don't show header of the graph (default: false)",
                    "no_axis: Don't show axis of the graph (default: false)",
                    "no_borders: Don't show border of the graph (default: false)",
                    "no_figures: Don't show figures of the graph (default: false)",
                    "lines: [{",
                    &[
                        "\tcolor: Colors of the Graph Line ({r: int, g: int, b: int} (0-255)) (default: nothing)",
                        "points: Points of the Graph Line ([{x: float, y: float}]"
                    ].join("\n\t\t\t"),
                    "}]"
                ].join("\n\t\t"),
                "}",
            ]
            .join("\n\t"),
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
        _input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::LabeledError> {
        let GraphConfig {
            ty,
            width,
            height,
            x_min,
            x_max,
            x_style,
            x_label,
            y_min,
            y_max,
            y_style,
            y_label,
            no_header,
            no_axis,
            no_borders,
            no_figures,
            lines,
        } = call.req(0)?;

        let ty = ty.unwrap_or_default();
        let width = width.unwrap_or(120);
        let height = height.unwrap_or(60);

        let x_style = x_style.unwrap_or_default();
        let x_label = x_label.unwrap_or_else(|| "X".into());

        let y_style = y_style.unwrap_or_default();
        let y_label = y_label.unwrap_or_else(|| "Y".into());

        let no_header = no_header.unwrap_or_default();
        let no_axis = no_axis.unwrap_or_default();
        let no_borders = no_borders.unwrap_or_default();
        let no_figures = no_figures.unwrap_or_default();

        let all_x_bits = lines
            .iter()
            .flat_map(|line| line.points.iter().map(|p| p.x.to_bits()))
            .collect::<Vec<_>>();

        let x_min = x_min
            .or_else(|| all_x_bits.iter().min().copied().map(f32::from_bits))
            .unwrap_or(f32::MIN);
        let x_max = x_max
            .or_else(|| all_x_bits.iter().max().copied().map(f32::from_bits))
            .unwrap_or(f32::MAX);

        let mut chart = match (y_min, y_max) {
            (Some(y_min), Some(y_max)) => {
                Chart::new_with_y_range(width, height, x_min, x_max, y_min, y_max)
            }
            _ => Chart::new(width, height, x_min, x_max),
        };

        let lines = lines
            .iter()
            .map(|GraphLineConfig { color, points }| {
                let shape_list = points
                    .iter()
                    .map(|GraphPoint { x, y }| (*x, *y))
                    .collect::<Vec<_>>();

                (color, shape_list)
            })
            .collect::<Vec<_>>();
        let lines_color_shape = lines
            .iter()
            .map(|(color, shape_list)| {
                let shape = match ty {
                    GraphType::Points => Shape::Points(shape_list),
                    GraphType::Lines => Shape::Lines(shape_list),
                    GraphType::Steps => Shape::Steps(shape_list),
                    GraphType::Bars => Shape::Bars(shape_list),
                };

                (color, shape)
            })
            .collect::<Vec<_>>();

        let prepared_chart = lines_color_shape.iter().fold(
            chart
                .x_axis_style(x_style.into())
                .y_axis_style(y_style.into()),
            |chart, (color, shape)| match color {
                Some(color) => chart.linecolorplot(shape, (*color).into()),
                None => chart.lineplot(shape),
            },
        );

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
        let chart_str = get_chart_str(prepared_chart);

        let resulting_str = (match no_header {
            true => vec![chart_str],
            false => vec![format!("{y_label}/{x_label}"), chart_str],
        })
        .join("\n");

        Ok(nu_protocol::PipelineData::Value(
            Value::string(resulting_str, call.head),
            None,
        ))
    }
}
