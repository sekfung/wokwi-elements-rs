use serde_json::Value;

use crate::{
    events::DomEvent,
    generated,
    render::Rendered,
    util::{bool_prop, numberish_prop, set_bool},
};

pub fn render(state: &Value) -> Rendered {
    let min = numberish_prop(state, "min", 0.0);
    let max = numberish_prop(state, "max", 1023.0);
    let value = numberish_prop(state, "value", 0.0);
    let step = numberish_prop(state, "step", 1.0);
    let start_degree = numberish_prop(state, "startDegree", -135.0);
    let end_degree = numberish_prop(state, "endDegree", 135.0);
    let percent = percent_from_min_max(value, min, max).clamp(0.0, 1.0);
    let knob_degree = (end_degree - start_degree) * percent + start_degree;

    let base = generated::render("wokwi-potentiometer").expect("potentiometer default render");
    let mut html = base.html.replace(
        r#"<input tabindex="0" type="range" class="hide-input" max="1023" min="0" value="0" step="1" aria-valuemin="0" aria-valuenow="0">"#,
        &format!(
            r#"<input tabindex="0" type="range" class="hide-input" data-on-input="input" max="{max}" min="{min}" value="{value}" step="{step}" aria-valuemin="{min}" aria-valuenow="{value}">"#
        ),
    );
    html = html.replace(
        r#"<svg role="slider" width="20mm" height="20mm" version="1.1" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" style="--knob-angle:-135deg;">"#,
        &format!(
            r#"<svg role="slider" width="20mm" height="20mm" version="1.1" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg" style="--knob-angle: {knob_degree}deg;" data-on-click="focus" data-on-mousedown="down" data-on-mousemove="rotate" data-on-mouseup="up" data-on-touchstart="down" data-on-touchmove="rotate" data-on-touchend="up">"#
        ),
    );

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    match action {
        "down" => {
            set_bool(state, "pressed", true);
            Vec::new()
        }
        "up" => {
            set_bool(state, "pressed", false);
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
        "rotate" => {
            if !bool_prop(state, "pressed", false) {
                return Vec::new();
            }
            let x = payload.get("svgX").and_then(Value::as_f64).unwrap_or(9.91);
            let y = payload.get("svgY").and_then(Value::as_f64).unwrap_or(8.18);
            let dx = 9.91 - x;
            let dy = 8.18 - y;
            let mut degree = (dy.atan2(dx) * 180.0 / std::f64::consts::PI).round();
            if degree < 0.0 {
                degree += 360.0;
            }
            degree -= 90.0;
            if dx > 0.0 && dy <= 0.0 && degree > 0.0 {
                degree -= 360.0;
            }
            let start = numberish_prop(state, "startDegree", -135.0);
            let end = numberish_prop(state, "endDegree", 135.0);
            let degree = degree.clamp(start.min(end), start.max(end));
            let percent = percent_from_min_max(degree, start, end);
            let min = numberish_prop(state, "min", 0.0);
            let max = numberish_prop(state, "max", 1023.0);
            update_value(state, percent * (max - min) + min)
        }
        _ => Vec::new(),
    }
}

fn percent_from_min_max(value: f64, min: f64, max: f64) -> f64 {
    if max == min {
        0.0
    } else {
        (value - min) / (max - min)
    }
}

fn update_value(state: &mut Value, value: f64) -> Vec<DomEvent> {
    let min = numberish_prop(state, "min", 0.0);
    let max = numberish_prop(state, "max", 1023.0);
    let step = numberish_prop(state, "step", 1.0);
    let clamped = value.clamp(min.min(max), min.max(max));
    let updated = if step > 0.0 {
        (clamped / step).round() * step
    } else {
        clamped
    };
    let updated = (updated * 100.0).round() / 100.0;
    if let Some(object) = state.as_object_mut() {
        object.insert("value".into(), Value::from(updated));
    }
    vec![DomEvent::input("input", Value::from(updated))]
}
