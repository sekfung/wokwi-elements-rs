use serde_json::{json, Value};

use crate::{events::DomEvent, generated, render::Rendered, util::format_js_number};

const DEGREES: [f64; 10] = [
    320.0, 56.0, 87.0, 115.0, 143.0, 173.0, 204.0, 232.0, 260.0, 290.0,
];

pub fn render(state: &Value) -> Rendered {
    let base = generated::render("wokwi-rotary-dialer").expect("dialer default render");
    let digit = state.get("_digit").and_then(Value::as_u64).unwrap_or(0);
    let angle = state.get("_angle").and_then(Value::as_f64).unwrap_or(0.0);
    let animated = state.get("_anim").and_then(Value::as_bool).unwrap_or(false);
    let mut html = base.html.replace(
        r##"<input tabindex="0" type="number" class="hide-input" value="">"##,
        &format!(r##"<input tabindex="0" type="number" class="hide-input" data-on-input="input" value="{digit}">"##),
    );
    for number in 0..=9u64 {
        let coords = match number {
            0 => (129, 243),
            1 => (230, 126),
            2 => (210, 79),
            3 => (168, 47),
            4 => (117, 41),
            5 => (72, 58),
            6 => (35, 97),
            7 => (28, 145),
            8 => (40, 194),
            _ => (78, 230),
        };
        html = html.replace(
            &format!(
                r##"<text x="{}" y="{}">{number}</text>"##,
                coords.0, coords.1
            ),
            &format!(
                r##"<text x="{}" y="{}" data-on-mouseup="dial:{number}">{number}</text>"##,
                coords.0, coords.1
            ),
        );
    }
    let class = if animated {
        " rotate-path dialer-anim "
    } else {
        " rotate-path "
    };
    let style = if animated {
        format!("--angle: {}deg;", format_js_number(angle))
    } else {
        String::new()
    };
    html = html.replace(
        r##"class=" rotate-path " style=""##,
        &format!(r##"class="{class}" style="{style}""##),
    );
    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, event: &str, payload: &Value) -> Vec<DomEvent> {
    let digit = if let Some(value) = event.strip_prefix("dial:") {
        value.parse::<u64>().ok()
    } else if event == "input" {
        payload
            .get("value")
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<u64>().ok())
    } else {
        None
    };
    let Some(digit) = digit.filter(|digit| *digit <= 9) else {
        return Vec::new();
    };
    if let Some(object) = state.as_object_mut() {
        object.insert("_digit".into(), Value::from(digit));
        object.insert("_angle".into(), Value::from(DEGREES[digit as usize]));
        object.insert("_anim".into(), Value::Bool(true));
    }
    vec![DomEvent::custom_detail(
        "dial-start",
        json!({ "digit": digit }),
    )]
}

pub fn invoke(state: &mut Value, method: &str, _args: &Value) -> bool {
    if method == "resetAnim" {
        if let Some(object) = state.as_object_mut() {
            object.insert("_anim".into(), Value::Bool(false));
            object.insert("_angle".into(), Value::from(0));
        }
        return true;
    }
    false
}
