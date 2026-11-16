use serde_json::Value;

use crate::{
    render::Rendered,
    util::{escape_html, number_prop, string_prop},
};

const GYR: [&str; 10] = [
    "#9eff3c", "#9eff3c", "#9eff3c", "#9eff3c", "#9eff3c", "#f1d73c", "#f1d73c", "#f1d73c",
    "#dc012d", "#dc012d",
];

const BCYR: [&str; 10] = [
    "#2c95fa", "#6cf9dc", "#6cf9dc", "#6cf9dc", "#6cf9dc", "#f1d73c", "#f1d73c", "#f1d73c",
    "#dc012d", "#dc012d",
];

pub fn render(state: &Value) -> Rendered {
    let color = string_prop(state, "color", "red");
    let off_color = escape_html(string_prop(state, "offColor", "#444"));
    let palette = match color {
        "GYR" => Some(GYR),
        "BCYR" => Some(BCYR),
        _ => None,
    };
    let values = state
        .get("values")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut segments = String::new();
    for index in 0..10 {
        let lit = values
            .get(index)
            .map(|value| value.as_f64().unwrap_or(0.0) != 0.0 || value.as_bool() == Some(true))
            .unwrap_or(false);
        let fill = if lit {
            palette
                .map(|colors| colors[index].to_owned())
                .unwrap_or_else(|| escape_html(color))
        } else {
            off_color.clone()
        };
        segments.push_str(&format!(
            r#"<rect x="2.5" y="{}" width="5" height="1.74" fill="{}"></rect>"#,
            0.4 + index as f64 * 2.54,
            fill
        ));
    }

    let html = format!(
        r##"<svg width="10.1mm" height="25.5mm" version="1.1" viewBox="0 0 10.1 25.5" xmlns="http://www.w3.org/2000/svg"><pattern id="pin-pattern" height="2.54" width="10.1" patternUnits="userSpaceOnUse"><circle cx="1.27" cy="1.27" r="0.5" fill="#aaa"></circle><circle cx="8.83" cy="1.27" r="0.5" fill="#aaa"></circle></pattern><path d="m1.4 0h8.75v25.5h-10.1v-24.2z"></path><rect width="10.1" height="25.4" fill="url(#pin-pattern)"></rect>{segments}</svg>"##
    );

    Rendered {
        html,
        styles: String::new(),
    }
}

#[allow(dead_code)]
fn _property_defaults_are_numbers(state: &Value) -> f64 {
    number_prop(state, "value", 0.0)
}
