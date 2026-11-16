use serde_json::Value;

use crate::{
    events::DomEvent,
    generated,
    render::Rendered,
    util::{bool_prop, numberish_prop, set_bool},
};

pub fn render(state: &Value) -> Rendered {
    let angle = numberish_prop(state, "angle", 0.0);
    let pressed = bool_prop(state, "pressed", false);
    let base = generated::render("wokwi-ky-040").expect("KY-040 default render");
    let mut html = base.html.replace(
        r##"transform="rotate(0, 36.244, 28.5)" class="handle ""##,
        &format!(
            r##"transform="rotate({angle}, 36.244, 28.5)" class="handle {}" data-on-mousedown="press" data-on-mouseup="release" data-on-mouseleave="release""##,
            if pressed { "active" } else { "" }
        ),
    );
    html = html.replace(
        r##"<g tabindex="0">"##,
        r##"<g tabindex="0" data-on-keydown="keydown" data-on-keyup="keyup">"##,
    );
    html = html.replacen(
        r##"<g class="arrow-container">"##,
        r##"<g class="arrow-container" data-on-mousedown="arrowStart:ccw" data-on-mouseup="arrowStop" data-on-mouseleave="arrowStop" data-on-click="ccw">"##,
        1,
    );
    html = html.replacen(
        r##"<g class="arrow-container">"##,
        r##"<g class="arrow-container" data-on-mousedown="arrowStart:cw" data-on-mouseup="arrowStop" data-on-mouseleave="arrowStop" data-on-click="cw">"##,
        1,
    );

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, event: &str, payload: &Value) -> Vec<DomEvent> {
    match event {
        "cw" => step(state, 1.0, "rotate-cw"),
        "ccw" => step(state, -1.0, "rotate-ccw"),
        "press" => {
            if bool_prop(state, "pressed", false) {
                Vec::new()
            } else {
                set_bool(state, "pressed", true);
                vec![DomEvent::input("button-press", Value::Null)]
            }
        }
        "release" => {
            if bool_prop(state, "pressed", false) {
                set_bool(state, "pressed", false);
                vec![DomEvent::input("button-release", Value::Null)]
            } else {
                Vec::new()
            }
        }
        "keydown" => match payload
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "ArrowUp" | "ArrowRight" => step(state, 1.0, "rotate-cw"),
            "ArrowDown" | "ArrowLeft" => step(state, -1.0, "rotate-ccw"),
            " " => action(state, "press", payload),
            _ => Vec::new(),
        },
        "keyup" if payload.get("key").and_then(Value::as_str) == Some(" ") => {
            action(state, "release", payload)
        }
        _ => Vec::new(),
    }
}

fn step(state: &mut Value, direction: f64, event_name: &str) -> Vec<DomEvent> {
    let angle = numberish_prop(state, "angle", 0.0);
    let step_size = numberish_prop(state, "stepSize", 18.0);
    let next = (angle + direction * step_size).rem_euclid(360.0);
    if let Some(object) = state.as_object_mut() {
        object.insert("angle".into(), Value::from(next));
    }
    vec![DomEvent::input(event_name, Value::Null)]
}
