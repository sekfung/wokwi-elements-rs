use serde_json::Value;

use crate::{
    events::DomEvent,
    generated,
    render::Rendered,
    util::{bool_prop, format_js_number, number_prop, set_bool},
};

pub fn render(state: &Value) -> Rendered {
    let x_value = number_prop(state, "xValue", 0.0);
    let y_value = number_prop(state, "yValue", 0.0);
    let pressed = bool_prop(state, "pressed", false);
    let base = generated::render("wokwi-analog-joystick").expect("joystick default render");
    let mut html = base.html.replace(
        r##"<circle id="knob" cx="13.6" cy="13.6" r="10.6" fill="url(#g-knob)" filter="url(#noise)" tabindex="0" transform="translate(0, 0)"></circle>"##,
        &format!(
            r##"<circle id="knob" cx="13.6" cy="13.6" r="10.6" fill="url(#g-knob)" filter="url(#noise)" tabindex="0" transform="translate({}, {})" data-on-keydown="keydown" data-on-keyup="keyup"></circle>"##,
            format_js_number(-2.5 * x_value),
            format_js_number(-2.5 * y_value)
        ),
    );

    for (old, action_down, action_up) in [
        (
            r##"<rect class="region" y="8.5" x="1" height="10" width="7"></rect>"##,
            "set:x:1",
            "clear:x",
        ),
        (
            r##"<rect class="region" y="1.38" x="7.9" height="7" width="10"></rect>"##,
            "set:y:1",
            "clear:y",
        ),
        (
            r##"<rect class="region" y="8.5" x="18" height="10" width="7"></rect>"##,
            "set:x:-1",
            "clear:x",
        ),
        (
            r##"<rect class="region" y="17" x="7.9" height="7" width="10"></rect>"##,
            "set:y:-1",
            "clear:y",
        ),
    ] {
        let tag = old.trim_end_matches("</rect>").trim_end_matches('>');
        html = html.replace(
            old,
            &format!(
                r##"{tag} data-on-mousedown="{action_down}" data-on-mouseup="{action_up}"></rect>"##
            ),
        );
    }

    html = html.replace(
        r##"<circle cx="13.6" cy="13.6" r="3" stroke="#aaa" class=""></circle>"##,
        &format!(
            r##"<circle cx="13.6" cy="13.6" r="3" stroke="#aaa" class="{}" data-on-mousedown="press" data-on-mouseup="release"></circle>"##,
            if pressed { "pressed" } else { "" }
        ),
    );

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    match action {
        "press" => {
            set_bool(state, "pressed", true);
            vec![DomEvent::input("button-press", Value::Null)]
        }
        "release" => {
            set_bool(state, "pressed", false);
            vec![DomEvent::input("button-release", Value::Null)]
        }
        "keydown" => keydown(state, payload),
        "keyup" => keyup(state, payload),
        _ if action.starts_with("set:") => {
            let mut parts = action.split(':');
            let _ = parts.next();
            let axis = parts.next().unwrap_or_default();
            let value = parts
                .next()
                .and_then(|value| value.parse::<f64>().ok())
                .unwrap_or(0.0);
            set_axis(state, axis, value);
            vec![DomEvent::input("input", Value::Null)]
        }
        "clear:x" => {
            set_axis(state, "x", 0.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        "clear:y" => {
            set_axis(state, "y", 0.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        _ => Vec::new(),
    }
}

fn keydown(state: &mut Value, payload: &Value) -> Vec<DomEvent> {
    match payload
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "ArrowUp" => {
            set_axis(state, "y", 1.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        "ArrowDown" => {
            set_axis(state, "y", -1.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        "ArrowLeft" => {
            set_axis(state, "x", 1.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        "ArrowRight" => {
            set_axis(state, "x", -1.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        " " | "Spacebar" => {
            set_bool(state, "pressed", true);
            vec![DomEvent::input("button-press", Value::Null)]
        }
        _ => Vec::new(),
    }
}

fn keyup(state: &mut Value, payload: &Value) -> Vec<DomEvent> {
    match payload
        .get("key")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "ArrowUp" | "ArrowDown" => {
            set_axis(state, "y", 0.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        "ArrowLeft" | "ArrowRight" => {
            set_axis(state, "x", 0.0);
            vec![DomEvent::input("input", Value::Null)]
        }
        " " | "Spacebar" => {
            set_bool(state, "pressed", false);
            vec![DomEvent::input("button-release", Value::Null)]
        }
        _ => Vec::new(),
    }
}

fn set_axis(state: &mut Value, axis: &str, value: f64) {
    let property = if axis == "x" { "xValue" } else { "yValue" };
    if let Some(object) = state.as_object_mut() {
        object.insert(property.into(), Value::from(value));
    }
}
