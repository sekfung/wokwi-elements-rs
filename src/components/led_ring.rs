use serde_json::{json, Value};

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{format_js_number, numberish_prop, string_prop},
};

const MM_TO_PIX: f64 = 3.78;
const PIN_HEIGHT: f64 = 3.0;
const PCB_WIDTH: f64 = 6.0;

pub fn render(state: &Value) -> Rendered {
    let pixels = numberish_prop(state, "pixels", 16.0).max(0.0) as usize;
    let pixel_spacing = numberish_prop(state, "pixelSpacing", 0.0);
    let background = string_prop(state, "background", "#363");
    let radius = ((pixel_spacing + 5.0) * pixels as f64) / 2.0 / std::f64::consts::PI + PCB_WIDTH;
    let width = radius * 2.0;
    let height = radius * 2.0 + PIN_HEIGHT;
    let pixel_colors = state.get("_pixelColors").and_then(Value::as_array);

    let mut pixel_elements = String::new();
    for index in 0..pixels {
        let angle = (index as f64 / pixels as f64) * 360.0;
        let color_style = pixel_colors
            .and_then(|colors| colors.get(index))
            .and_then(Value::as_object)
            .map(|color| {
                let red = numberish_prop(&Value::Object(color.clone()), "r", 0.0) * 255.0;
                let green = numberish_prop(&Value::Object(color.clone()), "g", 0.0) * 255.0;
                let blue = numberish_prop(&Value::Object(color.clone()), "b", 0.0) * 255.0;
                format!(
                    r#" style="fill: rgb({}, {}, {});""#,
                    format_js_number(red),
                    format_js_number(green),
                    format_js_number(blue)
                )
            })
            .unwrap_or_default();
        pixel_elements.push_str(&format!(
            r#"<rect class="pixel" x="{}" y="{}" width="5" height="5" fill="white" stroke="black" stroke-width="0.25" transform="rotate({} {} {})"{color_style}></rect>"#,
            format_js_number(radius - 2.5),
            format_js_number(PCB_WIDTH / 2.0 - 2.5),
            format_js_number(angle),
            format_js_number(radius),
            format_js_number(radius),
        ));
    }

    let html = format!(
        r##"<svg width="{}mm" height="{}mm" version="1.1" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg"><defs><pattern id="pin-pattern" height="2" width="2.54" patternUnits="userSpaceOnUse"><rect x="1.02" y="0" height="2" width="0.5" fill="#aaa"></rect></pattern></defs><rect fill="url(#pin-pattern)" height="{}" width="{}" transform="translate({}, {})"></rect><circle cx="{}" cy="{}" r="{}" fill="transparent" stroke-width="{}" stroke="{}"></circle>{pixel_elements}</svg>"##,
        format_js_number(width),
        format_js_number(height),
        format_js_number(width),
        format_js_number(height),
        format_js_number(PIN_HEIGHT + 1.0),
        format_js_number(4.0 * 2.54),
        format_js_number(radius - (4.0 * 2.54) / 2.0),
        format_js_number(radius * 2.0 - 1.0),
        format_js_number(radius),
        format_js_number(radius),
        format_js_number(radius - PCB_WIDTH / 2.0),
        format_js_number(PCB_WIDTH),
        background,
    );

    Rendered {
        html,
        styles: String::new(),
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let pixels = numberish_prop(state, "pixels", 16.0).max(0.0);
    let pixel_spacing = numberish_prop(state, "pixelSpacing", 0.0);
    let radius = ((pixel_spacing + 5.0) * pixels) / 2.0 / std::f64::consts::PI + PCB_WIDTH;
    let y = (radius * 2.0 + PIN_HEIGHT) * MM_TO_PIX;
    let center_x = radius * MM_TO_PIX;
    let spacing = 2.54 * MM_TO_PIX;
    vec![
        PinInfo {
            name: "GND".into(),
            x: center_x - 1.5 * spacing,
            y,
            number: None,
            description: None,
            signals: vec![json!({ "type": "power", "signal": "GND" })],
            no_breadboard: None,
        },
        PinInfo {
            name: "VCC".into(),
            x: center_x - 0.5 * spacing,
            y,
            number: None,
            description: None,
            signals: vec![json!({ "type": "power", "signal": "VCC" })],
            no_breadboard: None,
        },
        PinInfo {
            name: "DIN".into(),
            x: center_x + 0.5 * spacing,
            y,
            number: None,
            description: None,
            signals: vec![],
            no_breadboard: None,
        },
        PinInfo {
            name: "DOUT".into(),
            x: center_x + 1.5 * spacing,
            y,
            number: None,
            description: None,
            signals: vec![],
            no_breadboard: None,
        },
    ]
}

pub fn invoke(state: &mut Value, method: &str, args: &Value) -> bool {
    match method {
        "setPixel" => {
            let Some(args) = args.as_array() else {
                return false;
            };
            let pixel = args.first().and_then(Value::as_u64).unwrap_or(0) as usize;
            let Some(color) = args.get(1).cloned() else {
                return false;
            };
            let pixels = numberish_prop(state, "pixels", 16.0).max(0.0) as usize;
            if pixel >= pixels {
                return false;
            }
            let state_object = state.as_object_mut().expect("component state object");
            let colors = state_object
                .entry("_pixelColors")
                .or_insert_with(|| Value::Array(vec![]));
            let colors = colors.as_array_mut().expect("_pixelColors array");
            while colors.len() < pixels {
                colors.push(Value::Null);
            }
            colors[pixel] = color;
            true
        }
        "reset" => {
            if let Some(state_object) = state.as_object_mut() {
                state_object.insert("_pixelColors".into(), Value::Array(vec![]));
            }
            true
        }
        "setPixelColors" => {
            let Some(colors) = args.as_array() else {
                return false;
            };
            if let Some(object) = state.as_object_mut() {
                object.insert("_pixelColors".into(), Value::Array(colors.clone()));
            }
            true
        }
        "animationFrame" => {
            let time = args
                .as_array()
                .and_then(|values| values.first())
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let pixels = numberish_prop(state, "pixels", 16.0).max(0.0) as usize;
            let colors = (0..pixels)
                .map(|pixel| {
                    json!({
                        "r": pixel_value(pixel as f64 * 100.0 + time),
                        "g": pixel_value(pixel as f64 * 100.0 + time + 200.0),
                        "b": pixel_value(pixel as f64 * 100.0 + time + 400.0),
                    })
                })
                .collect();
            if let Some(object) = state.as_object_mut() {
                object.insert("_pixelColors".into(), Value::Array(colors));
            }
            true
        }
        _ => false,
    }
}

fn pixel_value(value: f64) -> f64 {
    let value = value % 2000.0;
    if value > 1000.0 {
        1.0 - (value % 1000.0) / 1000.0
    } else {
        (value % 1000.0) / 1000.0
    }
}
