use serde_json::{json, Value};

use crate::{
    events::DomEvent,
    generated,
    pins::PinInfo,
    render::Rendered,
    util::{bool_prop, format_js_number, numberish_prop, set_bool},
};

const MM_TO_PIX: f64 = 3.78;

pub fn render(state: &Value) -> Rendered {
    let travel = numberish_prop(state, "travelLength", 30.0);
    let value = numberish_prop(state, "value", 0.0);
    let min = numberish_prop(state, "min", 0.0);
    let max = numberish_prop(state, "max", 100.0);
    let step = numberish_prop(state, "step", 2.0);
    let tip_x = if max == min {
        -15.0
    } else {
        value / (max - min) * travel - 15.0
    };
    let base =
        generated::render("wokwi-slide-potentiometer").expect("slide potentiometer default render");
    let html = base
        .html
        .replace(
            r##"<input tabindex="0" type="range" class="hide-input" min="0" max="100" value="0" step="2" aria-valuemin="0" aria-valuenow="0" aria-valuemax="100">"##,
            &format!(
                r##"<input tabindex="0" type="range" class="hide-input" data-on-input="input" min="{min}" max="{max}" value="{value}" step="{step}" aria-valuemin="{min}" aria-valuenow="{value}" aria-valuemax="{max}">"##
            ),
        )
        .replace(
            r##"<svg height="29mm" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="55mm" viewBox="0 0 55 29">"##,
            &format!(
                r##"<svg height="29mm" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}mm" viewBox="0 0 {} 29">"##,
                format_js_number(travel + 25.0),
                format_js_number(travel + 25.0)
            ),
        )
        .replace(
            r##"<rect y="11" width="5" height="0.75" x="50"></rect>"##,
            &format!(r##"<rect y="11" width="5" height="0.75" x="{}"></rect>"##, format_js_number(travel + 20.0)),
        )
        .replace(
            r##"fill-rule="evenodd" width="45"></rect>"##,
            &format!(r##"fill-rule="evenodd" width="{}"></rect>"##, format_js_number(travel + 15.0)),
        )
        .replace(
            r##"<rect x="3.25" y="8" height="3" rx=".1" ry=".1" fill="#3f1e1e" width="38.5"></rect>"##,
            &format!(
                r##"<rect x="3.25" y="8" height="3" rx=".1" ry=".1" fill="#3f1e1e" width="{}"></rect>"##,
                format_js_number(travel + 8.5)
            ),
        )
        .replace(
            r##"<g transform="translate(43.375 9.5) rotate(45)">"##,
            &format!(
                r##"<g transform="translate({} 9.5) rotate(45)">"##,
                format_js_number(travel + 13.375)
            ),
        )
        .replace(
            r##"<g id="tip" transform="translate(-15 0)">"##,
            &format!(
                r##"<g id="tip" transform="translate({} 0)" data-on-mousedown="down" data-on-touchstart="down" data-on-touchmove="drag" data-on-touchend="up" data-on-keydown="keydown" data-on-keyup="keyup" data-on-click="focus">"##,
                format_js_number(tip_x)
            ),
        );

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let travel = numberish_prop(state, "travelLength", 30.0);
    vec![
        pin(
            "VCC",
            1.0,
            43.0,
            1,
            json!({ "type": "power", "signal": "VCC" }),
        ),
        pin(
            "SIG",
            1.0,
            63.0,
            2,
            json!({ "type": "analog", "channel": 0 }),
        ),
        pin(
            "GND",
            93.6 + travel * MM_TO_PIX,
            43.0,
            3,
            json!({ "type": "power", "signal": "GND" }),
        ),
    ]
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    match action {
        "down" => {
            set_bool(state, "isPressed", true);
            Vec::new()
        }
        "up" => {
            set_bool(state, "isPressed", false);
            Vec::new()
        }
        "input" => {
            let value = payload
                .get("value")
                .and_then(Value::as_str)
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0);
            update_value(state, value)
        }
        "drag" => {
            if !bool_prop(state, "isPressed", false) {
                return Vec::new();
            }
            let x = payload.get("svgX").and_then(Value::as_f64).unwrap_or(0.0);
            let travel = numberish_prop(state, "travelLength", 30.0);
            let min = numberish_prop(state, "min", 0.0);
            let max = numberish_prop(state, "max", 100.0);
            let value = ((x - 7.5) / (travel / (max - min))).round();
            update_value(state, value)
        }
        "keydown" if space_key(payload) => {
            set_bool(state, "isPressed", true);
            Vec::new()
        }
        "keyup" if space_key(payload) => {
            set_bool(state, "isPressed", false);
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn update_value(state: &mut Value, value: f64) -> Vec<DomEvent> {
    let min = numberish_prop(state, "min", 0.0);
    let max = numberish_prop(state, "max", 100.0);
    let value = value.clamp(min.min(max), min.max(max));
    if let Some(object) = state.as_object_mut() {
        object.insert("value".into(), Value::from(value));
    }
    vec![DomEvent::input("input", Value::from(value))]
}

fn pin(name: &str, x: f64, y: f64, number: u32, signal: Value) -> PinInfo {
    PinInfo {
        name: name.into(),
        x,
        y,
        number: Some(number),
        description: None,
        signals: vec![signal],
        no_breadboard: None,
    }
}

fn space_key(payload: &Value) -> bool {
    matches!(
        payload.get("key").and_then(Value::as_str),
        Some(" ") | Some("Spacebar")
    )
}
