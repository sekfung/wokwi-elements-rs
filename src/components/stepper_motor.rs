use serde_json::Value;

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{escape_html, format_js_number, numberish_prop, render_template, string_prop},
};

const MM_TO_PIX: f64 = 3.78;

struct StepperSpec {
    frame_size: f64,
    hole_radius: f64,
    shaft_radius: f64,
    corner_radius: f64,
    corner_offset: f64,
    body_radius: f64,
    text_size: f64,
    value_y: f64,
    units_y: f64,
}

pub fn render(state: &Value) -> Rendered {
    let size = numberish_prop(state, "size", 23.0).round() as i32;
    let spec = spec(size);
    let angle = numberish_prop(state, "angle", 0.0);
    let arrow = escape_html(string_prop(state, "arrow", ""));
    let value = escape_html(string_prop(state, "value", ""));
    let units = escape_html(string_prop(state, "units", ""));
    let half_frame = spec.frame_size / 2.0;
    let shaft_radius = spec.shaft_radius;
    let half_shaft = shaft_radius / 2.0;
    let r_off = (0.75 * shaft_radius * shaft_radius).sqrt();
    let html = render_template(
        include_str!("templates/stepper_motor.svg"),
        &[
            ("svgWidth", format_js_number(spec.frame_size + 1.0)),
            ("svgHeight", format_js_number(spec.frame_size + 5.0)),
            (
                "viewWidth",
                format_js_number((1.0 + spec.frame_size) * MM_TO_PIX),
            ),
            (
                "viewHeight",
                format_js_number((5.0 + spec.frame_size) * MM_TO_PIX),
            ),
            ("frameGradientX1", format_js_number(-spec.frame_size * 0.2)),
            ("frameGradientX2", format_js_number(spec.frame_size * 2.0)),
            ("bodyGradientX1", format_js_number(spec.frame_size * 0.1)),
            ("bodyGradientX2", format_js_number(spec.frame_size * 0.7)),
            ("frameSize", format_js_number(spec.frame_size)),
            ("cornerRadius", format_js_number(spec.corner_radius)),
            ("cornerOffset", format_js_number(spec.corner_offset)),
            (
                "frameSizeMinusCorner",
                format_js_number(spec.frame_size - spec.corner_offset),
            ),
            ("outerHoleRadius", format_js_number(spec.hole_radius * 1.1)),
            ("innerHoleRadius", format_js_number(spec.hole_radius * 0.9)),
            ("halfFrameMinus375", format_js_number(half_frame - 3.75)),
            ("halfFrame", format_js_number(half_frame)),
            ("bodyRadius", format_js_number(spec.body_radius)),
            ("angle", format_js_number(angle)),
            ("shaftRadius", format_js_number(shaft_radius)),
            ("arrowTip", format_js_number(half_frame - 3.0)),
            ("halfShaft", format_js_number(half_shaft)),
            ("rOff", format_js_number(r_off)),
            (
                "arrow",
                if arrow.is_empty() {
                    "transparent".into()
                } else {
                    arrow
                },
            ),
            ("valueY", format_js_number(spec.value_y)),
            ("unitsY", format_js_number(spec.units_y)),
            ("textSize", format_js_number(spec.text_size / MM_TO_PIX)),
            (
                "unitsTextSize",
                format_js_number(0.7 * spec.text_size / MM_TO_PIX),
            ),
            ("value", value),
            ("units", units),
        ],
    );
    Rendered {
        html,
        styles: String::new(),
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let size = numberish_prop(state, "size", 23.0).round() as i32;
    let spec = spec(size);
    let x_off = (spec.frame_size / 2.0 - 3.75) * MM_TO_PIX + 1.0;
    let y_off = (spec.frame_size + 5.0) * MM_TO_PIX;
    [("A-", 0.0), ("A+", 1.0), ("B+", 2.0), ("B-", 3.0)]
        .into_iter()
        .enumerate()
        .map(|(index, (name, offset))| PinInfo {
            name: name.into(),
            x: x_off + offset * 2.54 * MM_TO_PIX,
            y: y_off,
            number: Some(index as u32 + 1),
            description: None,
            signals: vec![],
            no_breadboard: None,
        })
        .collect()
}

fn spec(size: i32) -> StepperSpec {
    match size {
        8 => StepperSpec {
            frame_size: 20.4,
            hole_radius: 0.5,
            shaft_radius: 3.5,
            corner_radius: 2.5,
            corner_offset: 2.5,
            body_radius: 7.7,
            text_size: 10.0,
            value_y: 16.5,
            units_y: 19.7,
        },
        11 => StepperSpec {
            frame_size: 28.2,
            hole_radius: 1.25,
            shaft_radius: 5.0,
            corner_radius: 2.5,
            corner_offset: 2.5,
            body_radius: 11.0,
            text_size: 12.0,
            value_y: 21.5,
            units_y: 24.0,
        },
        14 => StepperSpec {
            frame_size: 35.2,
            hole_radius: 1.5,
            shaft_radius: 5.0,
            corner_radius: 4.5,
            corner_offset: 4.5,
            body_radius: 11.0,
            text_size: 14.0,
            value_y: 26.0,
            units_y: 32.0,
        },
        17 => StepperSpec {
            frame_size: 42.3,
            hole_radius: 1.5,
            shaft_radius: 5.0,
            corner_radius: 5.0,
            corner_offset: 5.5,
            body_radius: 14.0,
            text_size: 16.0,
            value_y: 30.5,
            units_y: 33.7,
        },
        34 => StepperSpec {
            frame_size: 86.0,
            hole_radius: 3.25,
            shaft_radius: 14.0,
            corner_radius: 3.25,
            corner_offset: 8.4,
            body_radius: 36.5,
            text_size: 32.0,
            value_y: 68.0,
            units_y: 75.0,
        },
        _ => StepperSpec {
            frame_size: 57.3,
            hole_radius: 2.5,
            shaft_radius: 6.35,
            corner_radius: 5.0,
            corner_offset: 5.5,
            body_radius: 19.5,
            text_size: 24.0,
            value_y: 41.0,
            units_y: 46.0,
        },
    }
}
