use serde_json::{json, Value};

use crate::{
    events::DomEvent,
    pins::PinInfo,
    render::Rendered,
    util::{escape_html, format_js_number, render_template, string_prop},
};

const ROW_POSITIONS: [f64; 4] = [10.7, 25.0, 39.3, 53.6];
const COLUMN_POSITIONS: [f64; 4] = [7.0, 22.0, 37.0, 52.0];
const DEFAULT_KEYS: [&str; 16] = [
    "1", "2", "3", "A", "4", "5", "6", "B", "7", "8", "9", "C", "*", "0", "#", "D",
];

pub fn render(state: &Value) -> Rendered {
    let four_columns = string_prop(state, "columns", "4") != "3";
    let column_width = 15.0;
    let width = if four_columns {
        70.336
    } else {
        70.336 - column_width
    };
    let connector = state
        .get("connector")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let height = 76.0 + if connector { 15.0 } else { 0.0 };
    let keys = keys(state);
    let pressed = state
        .get("_pressedKeys")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut blue = String::new();
    let mut red = String::new();
    for row in 0..4 {
        for column in 0..4 {
            if !four_columns && column == 3 {
                continue;
            }
            let text = keys.get(row * 4 + column).cloned().unwrap_or_default();
            let numeric = text.parse::<f64>().is_ok();
            let key = render_key(row, column, &text, numeric, &pressed);
            if numeric {
                blue.push_str(&key);
            } else {
                red.push_str(&key);
            }
        }
    }
    let connector_markup = if connector {
        let connector_width = if four_columns { 2.54 * 8.0 } else { 2.54 * 7.0 };
        format!(
            r##"<g transform="translate({}, 76)"><rect width="{}" height="8" fill="url(#wires)"></rect><rect width="10.16" height="8" fill="url(#wires-marks)"></rect><rect y="8" width="{}" height="7" fill="#333"></rect><rect transform="translate(0, 12)" width="{}" height="2.54" fill="url(#pins-female)"></rect></g>"##,
            format_js_number((width - connector_width) / 2.0),
            format_js_number(connector_width),
            format_js_number(connector_width),
            format_js_number(connector_width)
        )
    } else {
        String::new()
    };
    let html = render_template(
        include_str!("templates/membrane_keypad.svg"),
        &[
            ("width", format_js_number(width)),
            ("height", format_js_number(height)),
            (
                "innerWidth",
                format_js_number(if four_columns {
                    65.0
                } else {
                    65.0 - column_width
                }),
            ),
            ("connector", connector_markup),
            ("blueKeys", blue),
            ("redKeys", red),
        ],
    );
    Rendered {
        html,
        styles: MEMBRANE_STYLES.into(),
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let three = string_prop(state, "columns", "4") == "3";
    let xs = if three {
        [76.5, 86.0, 95.75, 105.25, 115.0, 124.5, 134.0, 0.0]
    } else {
        [100.0, 110.0, 119.5, 129.0, 138.5, 148.0, 157.75, 167.5]
    };
    let mut pins = Vec::new();
    for row in 0..4 {
        pins.push(pin(&format!("R{}", row + 1), xs[row], 338.0));
    }
    for col in 0..if three { 3 } else { 4 } {
        pins.push(pin(&format!("C{}", col + 1), xs[4 + col], 338.0));
    }
    pins
}

pub fn action(state: &mut Value, event: &str, payload: &Value) -> Vec<DomEvent> {
    if (event == "up" || event == "keyup")
        && payload.get("key").and_then(Value::as_str) == Some("Shift")
    {
        let pressed = state
            .get("_pressedKeys")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let keys = keys(state);
        let events = pressed
            .iter()
            .filter_map(Value::as_str)
            .filter_map(|key| {
                let index = keys.iter().position(|candidate| candidate == key)?;
                Some(DomEvent::custom_detail(
                    "button-release",
                    json!({ "key": key, "row": index / 4, "column": index % 4 }),
                ))
            })
            .collect();
        set_pressed(state, Vec::new());
        return events;
    }
    let Some((key, row, column)) = resolve_key(state, event, payload) else {
        return Vec::new();
    };
    let mut pressed = state
        .get("_pressedKeys")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    match event {
        "down" | "mousedown" | "touchstart" | "keydown" => {
            if pressed.iter().any(|v| v.as_str() == Some(&key)) {
                return Vec::new();
            }
            pressed.push(Value::String(key.clone()));
            set_pressed(state, pressed);
            vec![DomEvent::custom_detail(
                "button-press",
                json!({"key":key,"row":row,"column":column}),
            )]
        }
        "up" | "mouseup" | "touchend" | "blur" | "keyup" => {
            pressed.retain(|v| v.as_str() != Some(&key));
            set_pressed(state, pressed);
            vec![DomEvent::custom_detail(
                "button-release",
                json!({"key":key,"row":row,"column":column}),
            )]
        }
        _ => Vec::new(),
    }
}

fn render_key(row: usize, column: usize, text: &str, numeric: bool, pressed: &[Value]) -> String {
    let key_name = escape_html(&text.to_uppercase());
    let label = escape_html(text);
    let class = if numeric { "blue-key" } else { "red-key" };
    let active = pressed.iter().any(|v| v.as_str() == Some(text));
    format!(
        r##"<g transform="translate({} {})" tabindex="0" class="{class}{}" data-key-name="{key_name}" data-on-mousedown="down" data-on-mouseup="up" data-on-touchstart="down" data-on-touchend="up" data-on-blur="blur" data-on-keydown="keydown" data-on-keyup="keyup"><rect width="11.2" height="11" rx="1.4" ry="1.4" stroke="#b1b5b9" stroke-width=".75"></rect><text x="5.6" y="8.1">{label}</text></g>"##,
        format_js_number(COLUMN_POSITIONS[column]),
        format_js_number(ROW_POSITIONS[row]),
        if active { " pressed" } else { "" }
    )
}

fn keys(state: &Value) -> Vec<String> {
    state
        .get("keys")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|| DEFAULT_KEYS.iter().map(|key| (*key).into()).collect())
}

fn resolve_key(state: &Value, event: &str, payload: &Value) -> Option<(String, usize, usize)> {
    if (event == "keydown" || event == "keyup")
        && matches!(
            payload.get("key").and_then(Value::as_str),
            Some(" ") | Some("Spacebar")
        )
    {
        if let Some(key) = payload.get("dataKey").and_then(Value::as_str) {
            let keys = keys(state);
            let index = keys
                .iter()
                .position(|candidate| candidate.eq_ignore_ascii_case(key))?;
            return Some((key.to_owned(), index / 4, index % 4));
        }
    }
    let key = if event == "keydown" || event == "keyup" {
        payload
            .get("key")
            .and_then(Value::as_str)
            .map(str::to_uppercase)?
    } else {
        payload.get("dataKey").and_then(Value::as_str)?.to_owned()
    };
    let keys = keys(state);
    let index = keys
        .iter()
        .position(|candidate| candidate == &key || candidate.to_uppercase() == key)?;
    Some((key, index / 4, index % 4))
}

fn set_pressed(state: &mut Value, pressed: Vec<Value>) {
    if let Some(object) = state.as_object_mut() {
        object.insert("_pressedKeys".into(), Value::Array(pressed));
    }
}

fn pin(name: &str, x: f64, y: f64) -> PinInfo {
    PinInfo {
        name: name.into(),
        x,
        y,
        number: None,
        description: None,
        signals: vec![],
        no_breadboard: None,
    }
}

const MEMBRANE_STYLES: &str = r#"text { fill: #dfe2e5; user-select: none; } g[tabindex] { cursor: pointer; } .blue-key.pressed { fill: #4e50d7; } .red-key.pressed { fill: #ab040b; } .blue-key.pressed text, .red-key.pressed text { fill: white; stroke: none; }"#;
