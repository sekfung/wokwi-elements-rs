use std::sync::OnceLock;

use serde_json::{json, Value};

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{
        bool_prop, escape_html, format_js_number, numberish_prop, render_template, string_prop,
    },
};

const MM_TO_PIX: f64 = 3.78;
const STYLES: &str = r#"
.cursor-blink { animation: cursor-blink; }
@keyframes cursor-blink { from { opacity: 0; } 25% { opacity: 1; } 75% { opacity: 1; } to { opacity: 0; } }
"#;

pub fn render(tag: &str, state: &Value) -> Rendered {
    let (cols, rows) = dimensions(tag);
    let color = escape_html(string_prop(state, "color", "black"));
    let background_name = string_prop(state, "background", "green");
    let background = match background_name {
        "green" => "#6cb201".to_owned(),
        "blue" => "#000eff".to_owned(),
        _ => "[object Object]".to_owned(),
    };
    let panel_height = rows as f64 * 5.75;
    let panel_width = cols as f64 * 3.5125;
    let screen_only = bool_prop(state, "screenOnly", false);
    let width = if screen_only {
        panel_width
    } else {
        panel_width + 23.8
    };
    let height = if screen_only {
        panel_height
    } else {
        panel_height + 24.5
    };
    let view_box = if screen_only {
        format!(
            "12.45 12.55 {} {}",
            format_js_number(panel_width),
            format_js_number(panel_height)
        )
    } else {
        format!(
            "0 0 {} {}",
            format_js_number(width),
            format_js_number(height)
        )
    };
    let pins_mode = string_prop(state, "pins", "full");
    let pins = match pins_mode {
        "i2c" => i2c_pins(),
        "full" => full_pins(panel_height),
        _ => String::new(),
    };
    let characters = characters(state);
    let font = font(state);
    let text_path = text_path(&characters, &font, cols);
    let cursor = cursor_markup(state, cols, rows, &color);
    let html = render_template(
        include_str!("templates/lcd.svg"),
        &[
            ("width", format_js_number(width)),
            ("height", format_js_number(height)),
            ("viewBox", view_box),
            ("panelWidthPlus15", format_js_number(panel_width + 15.0)),
            ("panelHeightPlus137", format_js_number(panel_height + 13.7)),
            ("panelWidthPlus98", format_js_number(panel_width + 9.8)),
            ("panelHeightPlus45", format_js_number(panel_height + 4.5)),
            ("background", background),
            (
                "darken",
                format_js_number(if bool_prop(state, "backlight", true) {
                    0.0
                } else {
                    0.5
                }),
            ),
            ("pins", pins),
            ("panelWidth", format_js_number(panel_width)),
            ("panelHeight", format_js_number(panel_height)),
            ("textPath", text_path),
            ("color", color),
            ("cursor", cursor),
        ],
    );
    Rendered {
        html,
        styles: STYLES.into(),
    }
}

pub fn pin_info(tag: &str, state: &Value) -> Vec<PinInfo> {
    let (_, rows) = dimensions(tag);
    let panel_height = rows as f64 * 5.75;
    let y = 87.5 + panel_height * MM_TO_PIX;
    if string_prop(state, "pins", "full") == "i2c" {
        return vec![
            pin(
                "GND",
                4.0,
                32.0,
                1,
                json!({ "type": "power", "signal": "GND" }),
            ),
            pin(
                "VCC",
                4.0,
                41.5,
                2,
                json!({ "type": "power", "signal": "VCC" }),
            ),
            pin(
                "SDA",
                4.0,
                51.0,
                3,
                json!({ "type": "i2c", "signal": "SDA", "bus": 0 }),
            ),
            pin(
                "SCL",
                4.0,
                60.5,
                4,
                json!({ "type": "i2c", "signal": "SCL", "bus": 0 }),
            ),
        ];
    }
    [
        ("VSS", 32.0, json!({"type":"power","signal":"GND"})),
        ("VDD", 41.5, json!({"type":"power","signal":"VCC"})),
        ("V0", 51.5, Value::Null),
        ("RS", 60.5, Value::Null),
        ("RW", 70.5, Value::Null),
        ("E", 80.0, Value::Null),
        ("D0", 89.5, Value::Null),
        ("D1", 99.5, Value::Null),
        ("D2", 109.0, Value::Null),
        ("D3", 118.5, Value::Null),
        ("D4", 128.0, Value::Null),
        ("D5", 137.5, Value::Null),
        ("D6", 147.0, Value::Null),
        ("D7", 156.5, Value::Null),
        ("A", 166.5, Value::Null),
        ("K", 176.0, Value::Null),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (name, x, signal))| pin(name, x, y, index as u32 + 1, signal))
    .collect()
}

fn dimensions(tag: &str) -> (usize, usize) {
    if tag == "wokwi-lcd2004" {
        (20, 4)
    } else {
        (16, 2)
    }
}

fn characters(state: &Value) -> Vec<u8> {
    if let Some(text) = state
        .get("text")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
    {
        return text.chars().map(|c| c as u32 as u8).collect();
    }
    state
        .get("characters")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect()
        })
        .unwrap_or_default()
}

fn font(state: &Value) -> Vec<u8> {
    if let Some(values) = state
        .get("font")
        .and_then(Value::as_array)
        .filter(|values| values.len() >= 2048)
    {
        return values
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();
    }
    static DEFAULT_FONT: OnceLock<Vec<u8>> = OnceLock::new();
    DEFAULT_FONT
        .get_or_init(|| serde_json::from_str(include_str!("lcd_font_a00.json")).expect("LCD font"))
        .clone()
}

fn text_path(characters: &[u8], font: &[u8], cols: usize) -> String {
    let mut path = String::new();
    for (i, character) in characters.iter().enumerate() {
        let char_x = (i % cols) as f64 * 3.55;
        let char_y = (i / cols) as f64 * 5.95;
        for py in 0..8 {
            let row = font.get(*character as usize * 8 + py).copied().unwrap_or(0);
            for px in 0..5 {
                if row & (1 << px) != 0 {
                    path.push_str(&format!(
                        "M {:.2} {:.2}h0.55v0.65h-0.55Z ",
                        char_x + px as f64 * 0.6,
                        char_y + py as f64 * 0.7
                    ));
                }
            }
        }
    }
    path
}

fn cursor_markup(state: &Value, cols: usize, rows: usize, color: &str) -> String {
    let cursor_x = numberish_prop(state, "cursorX", 0.0) as usize;
    let cursor_y = numberish_prop(state, "cursorY", 0.0) as usize;
    if cursor_x >= cols || cursor_y >= rows {
        return String::new();
    }
    let x = 12.45 + cursor_x as f64 * 3.55;
    let y = 12.55 + cursor_y as f64 * 5.95;
    let mut output = String::new();
    if bool_prop(state, "blink", false) {
        output.push_str(&format!(r##"<rect x="{}" y="{}" width="2.95" height="5.55" fill="{}"><animate attributeName="opacity" values="0;0;0;0;1;1;0;0;0;0" dur="1s" fill="freeze" repeatCount="indefinite"></animate></rect>"##, format_js_number(x), format_js_number(y), color));
    }
    if bool_prop(state, "cursor", false) {
        output.push_str(&format!(
            r##"<rect x="{}" y="{}" width="2.95" height="0.65" fill="{}"></rect>"##,
            format_js_number(x),
            format_js_number(y + 4.9),
            color
        ));
    }
    output
}

fn i2c_pins() -> String {
    r##"<rect x="7.55" y="-2.5" height="2.5" width="10.16" fill="url(#pins)" transform="rotate(90)"></rect><text fill="white" font-size="1.5px" font-family="monospace"><tspan y="6.8" x="0.7" fill="white">1</tspan><tspan y="8.9" x="2.3" fill="white">GND</tspan><tspan y="11.4" x="2.3" fill="white">VCC</tspan><tspan y="14" x="2.3" fill="white">SDA</tspan><tspan y="16.6" x="2.3" fill="white">SCL</tspan></text>"##.into()
}

fn full_pins(panel_height: f64) -> String {
    let y = panel_height + 21.1;
    format!(
        r##"<g transform="translate(0, {})"><rect x="7.55" y="1" height="2.5" width="40.64" fill="url(#pins)"></rect><text fill="white" font-size="1.5px" font-family="monospace"><tspan x="6" y="2.7">1</tspan><tspan x="7.2" y="0.7">VSS</tspan><tspan x="9.9" y="0.7">VDD</tspan><tspan x="12.7" y="0.7">V0</tspan><tspan x="15.2" y="0.7">RS</tspan><tspan x="17.8" y="0.7">RW</tspan><tspan x="20.8" y="0.7">E</tspan><tspan x="22.7" y="0.7">D0</tspan><tspan x="25.3" y="0.7">D1</tspan><tspan x="27.9" y="0.7">D2</tspan><tspan x="30.4" y="0.7">D3</tspan><tspan x="33" y="0.7">D4</tspan><tspan x="35.6" y="0.7">D5</tspan><tspan x="38.2" y="0.7">D6</tspan><tspan x="40.8" y="0.7">D7</tspan><tspan x="43.6" y="0.7">A</tspan><tspan x="46.2" y="0.7">K</tspan><tspan x="48" y="2.7">16</tspan></text></g>"##,
        format_js_number(y)
    )
}

fn pin(name: &str, x: f64, y: f64, number: u32, signal: Value) -> PinInfo {
    PinInfo {
        name: name.into(),
        x,
        y,
        number: Some(number),
        description: None,
        signals: if signal.is_null() {
            vec![]
        } else {
            vec![signal]
        },
        no_breadboard: None,
    }
}
