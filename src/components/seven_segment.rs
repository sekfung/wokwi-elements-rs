use serde_json::Value;

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{bool_prop, escape_html, number_prop, string_prop},
};

const MM_TO_PIX: f64 = 3.78;

const STYLES: &str = r#"
polygon {
  transform: scale(0.9);
  transform-origin: 50% 50%;
  transform-box: fill-box;
}
"#;

pub fn render(state: &Value) -> Rendered {
    let digits = number_prop(state, "digits", 1.0).max(1.0) as usize;
    let colon = bool_prop(state, "colon", false);
    let pins = string_prop(state, "pins", "top");
    let color = escape_html(string_prop(state, "color", "red"));
    let off_color = escape_html(string_prop(state, "offColor", "#444"));
    let values = values(state);
    let y_offset = if pins == "extend" { 2.0 } else { 0.0 };
    let width = 12.55 * digits as f64;
    let height = if pins == "extend" { 23.0 } else { 22.0 };

    let mut digit_shapes = String::new();
    for index in 0..digits {
        digit_shapes.push_str(&render_digit(
            3.5 + index as f64 * 12.7,
            index * 8,
            y_offset,
            &values,
            &color,
            &off_color,
        ));
    }

    let colon_shape = if colon {
        render_colon(
            digits,
            y_offset,
            bool_prop(state, "colonValue", false),
            &color,
            &off_color,
        )
    } else {
        String::new()
    };
    let pin_pattern = if pins == "extend" {
        r##"<rect x="1.02" y="0" height="2" width="0.5" fill="#aaa" />"##
    } else {
        r##"<circle cx="1.27" cy="1" r="0.5" fill="#aaa" />"##
    };
    let pins_shape = if pins != "none" && pins != "extend" {
        render_pins(digits, pins)
    } else if pins == "extend" {
        render_pins(digits, pins)
    } else {
        String::new()
    };

    Rendered {
        html: format!(
            r#"<svg width="{width}mm" height="{height}mm" version="1.1" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg"><defs><pattern id="pin-pattern" height="2" width="2.54" patternUnits="userSpaceOnUse">{pin_pattern}</pattern></defs><rect x="0" y="{y_offset}" width="{width}" height="20.5" />{digit_shapes}{colon_shape}{pins_shape}</svg>"#
        ),
        styles: STYLES.into(),
    }
}

fn values(state: &Value) -> Vec<bool> {
    state
        .get("values")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(|value| {
                    value
                        .as_bool()
                        .or_else(|| value.as_f64().map(|number| number != 0.0))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn render_digit(
    x: f64,
    start_index: usize,
    y_offset: f64,
    values: &[bool],
    color: &str,
    off_color: &str,
) -> String {
    let fill = |index: usize| {
        if values.get(start_index + index).copied().unwrap_or(false) {
            color
        } else {
            off_color
        }
    };
    format!(
        r#"<g transform="skewX(-8) translate({x}, {}) scale(0.81)"><polygon points="2 0 8 0 9 1 8 2 2 2 1 1" fill="{}" /><polygon points="10 2 10 8 9 9 8 8 8 2 9 1" fill="{}" /><polygon points="10 10 10 16 9 17 8 16 8 10 9 9" fill="{}" /><polygon points="8 18 2 18 1 17 2 16 8 16 9 17" fill="{}" /><polygon points="0 16 0 10 1 9 2 10 2 16 1 17" fill="{}" /><polygon points="0 8 0 2 1 1 2 2 2 8 1 9" fill="{}" /><polygon points="2 8 8 8 9 9 8 10 2 10 1 9" fill="{}" /></g><circle cx="{}" cy="{}" r="0.89" fill="{}" />"#,
        y_offset + 2.4,
        fill(0),
        fill(1),
        fill(2),
        fill(3),
        fill(4),
        fill(5),
        fill(6),
        x + 7.4,
        y_offset + 16.0,
        fill(7),
    )
}

fn render_colon(digits: usize, y_offset: f64, lit: bool, color: &str, off_color: &str) -> String {
    let fill = if lit { color } else { off_color };
    let colon_position = 1.5 + 12.7 * ((digits as f64 / 2.0).round());
    format!(
        r#"<g transform="skewX(-8)" fill="{fill}"><circle cx="{colon_position}" cy="{}" r="0.89" /><circle cx="{colon_position}" cy="{}" r="0.89" /></g>"#,
        y_offset + 5.75,
        y_offset + 13.25,
    )
}

fn render_pins(digits: usize, pins: &str) -> String {
    let (start_x, bottom_y, cols) = pin_positions(digits, pins);
    format!(
        r#"<g fill="url(#pin-pattern)" transform="translate({start_x}, 0)"><rect height="2" width="{}" /><rect height="2" width="{}" transform="translate(0, {bottom_y})" /></g>"#,
        cols as f64 * 2.54,
        cols as f64 * 2.54,
    )
}

fn pin_positions(digits: usize, pins: &str) -> (f64, f64, usize) {
    let num_pins: usize = match digits {
        4 => 14,
        3 => 12,
        _ => 10,
    };
    let cols = (num_pins + 1) / 2;
    (
        (12.55 * digits as f64 - cols as f64 * 2.54) / 2.0,
        if pins == "extend" { 21.0 } else { 18.0 },
        cols,
    )
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let digits = number_prop(state, "digits", 1.0) as usize;
    let pins = string_prop(state, "pins", "top");
    let names: &[(&str, u32, &str)] = match digits {
        4 => &[
            ("A", 13, "Segment A"),
            ("B", 9, "Segment B"),
            ("C", 4, "Segment C"),
            ("D", 2, "Segment D"),
            ("E", 1, "Segment E"),
            ("F", 12, "Segment F"),
            ("G", 5, "Segment G"),
            ("DP", 3, "Decimal Point"),
            ("DIG1", 14, "Digit 1 Common"),
            ("DIG2", 11, "Digit 2 Common"),
            ("DIG3", 10, "Digit 3 Common"),
            ("DIG4", 6, "Digit 4 Common"),
            ("COM", 7, "Common pin"),
            ("CLN", 8, "Colon"),
        ],
        3 => &[
            ("A", 11, "Segment A"),
            ("B", 7, "Segment B"),
            ("C", 4, "Segment C"),
            ("D", 2, "Segment D"),
            ("E", 1, "Segment E"),
            ("F", 10, "Segment F"),
            ("G", 5, "Segment G"),
            ("DP", 3, "Decimal Point"),
            ("DIG1", 12, "Digit 1 Common"),
            ("DIG2", 9, "Digit 2 Common"),
            ("DIG3", 8, "Digit 3 Common"),
        ],
        2 => &[
            ("DIG1", 8, "Digit 1 Common"),
            ("DIG2", 7, "Digit 2 Common"),
            ("A", 10, "Segment A"),
            ("B", 9, "Segment B"),
            ("C", 1, "Segment C"),
            ("D", 4, "Segment D"),
            ("E", 3, "Segment E"),
            ("F", 6, "Segment F"),
            ("G", 5, "Segment G"),
            ("DP", 2, "Decimal Point"),
        ],
        _ => &[
            ("COM.1", 3, "Common"),
            ("COM.2", 8, "Common"),
            ("A", 7, "Segment A"),
            ("B", 6, "Segment B"),
            ("C", 4, "Segment C"),
            ("D", 2, "Segment D"),
            ("E", 1, "Segment E"),
            ("F", 9, "Segment F"),
            ("G", 10, "Segment G"),
            ("DP", 5, "Decimal Point"),
        ],
    };

    names
        .iter()
        .map(|(name, number, description)| {
            let (x, y) = pin_xy(*number, digits, pins);
            PinInfo {
                name: (*name).into(),
                x,
                y,
                number: Some(*number),
                description: Some((*description).into()),
                signals: vec![],
                no_breadboard: None,
            }
        })
        .collect()
}

fn pin_xy(number: u32, digits: usize, pins: &str) -> (f64, f64) {
    let (start_x, bottom_y, cols) = pin_positions(digits, pins);
    let col = (number as usize - 1) % cols;
    let row = 1 - ((number as usize - 1) / cols);
    let x_offset = 1.27;
    let x = start_x + x_offset + if row == 1 { col } else { cols - col - 1 } as f64 * 2.54;
    let y = if pins == "top" {
        if row == 1 {
            bottom_y + 1.0
        } else {
            1.0
        }
    } else if row == 1 {
        bottom_y + 2.0
    } else {
        0.0
    };
    (x * MM_TO_PIX, y * MM_TO_PIX)
}
