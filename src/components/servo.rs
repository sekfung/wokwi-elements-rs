use serde_json::Value;

use crate::{
    generated,
    render::Rendered,
    util::{escape_html, format_js_number, number_prop, string_prop},
};

pub fn render(state: &Value) -> Rendered {
    let angle = number_prop(state, "angle", 0.0);
    let horn = string_prop(state, "horn", "single");
    let horn_color = escape_html(string_prop(state, "hornColor", "#ccc"));
    let paths: Value = serde_json::from_str(include_str!("horn_paths.json"))
        .expect("generated servo horn paths must be valid");
    let path = paths
        .get(horn)
        .and_then(Value::as_str)
        .or_else(|| paths.get("single").and_then(Value::as_str))
        .expect("single servo horn path");

    let base = generated::render("wokwi-servo").expect("servo default render");
    let start = "<path fill=\"#ccc\" transform=\"translate(91.467 59.773) rotate(0) translate(-91.467 -59.773)\"";
    let html = replace_element(
        &base.html,
        start,
        "</path>",
        &format!(
            "<path fill=\"{horn_color}\" transform=\"translate(91.467 59.773) rotate({}) translate(-91.467 -59.773)\" d=\"{path}\"></path>",
            format_js_number(angle)
        ),
    );

    Rendered {
        html,
        styles: base.styles,
    }
}

fn replace_element(html: &str, start: &str, end: &str, replacement: &str) -> String {
    let Some(start_index) = html.find(start) else {
        return html.to_owned();
    };
    let Some(relative_end) = html[start_index..].find(end) else {
        return html.to_owned();
    };
    let end_index = start_index + relative_end + end.len();
    let mut output = String::with_capacity(html.len() + replacement.len());
    output.push_str(&html[..start_index]);
    output.push_str(replacement);
    output.push_str(&html[end_index..]);
    output
}
