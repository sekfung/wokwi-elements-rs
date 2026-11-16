use serde_json::Value;

use crate::{events::DomEvent, generated, render::Rendered, util::bool_prop};

const X_POSITIONS: [f64; 8] = [0.0, 9.6, 19.4, 29.1, 38.5, 48.1, 57.7, 67.3];

pub fn render(state: &Value) -> Rendered {
    let base = generated::render("wokwi-dip-switch-8").expect("dip switch default render");
    let values = state
        .get("values")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut switches = String::new();

    for (index, x) in X_POSITIONS.iter().enumerate() {
        let on = values
            .get(index)
            .map(|value| value.as_f64().unwrap_or(0.0) != 0.0 || value.as_bool() == Some(true))
            .unwrap_or(false);
        let y = if on { -7.2 } else { 0.0 };
        switches.push_str(&format!(
            r##"<rect data-on-click="toggle:{index}" y="21.2" width="5.8168" height="13" x="{}"></rect><use data-on-click="toggle:{index}" xlink:href="#switch" x="{x}" y="{}"></use>"##,
            x + 4.693,
            crate::util::format_js_number(y),
        ));
    }

    let anchor = r##"<g fill="#917c6f" stroke-width=".77094">"##;
    let html = replace_group_contents(&base.html, anchor, &switches).replacen(
        "<svg tabindex=\"0\"",
        "<svg tabindex=\"0\" data-on-keydown=\"keydown\"",
        1,
    );
    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, action: &str, _payload: &Value) -> Vec<DomEvent> {
    if action == "keydown" {
        let Some(key) = _payload.get("key").and_then(Value::as_str) else {
            return Vec::new();
        };
        let Some(index) = key
            .parse::<usize>()
            .ok()
            .filter(|index| (1..=8).contains(index))
        else {
            return Vec::new();
        };
        return toggle(state, index - 1);
    }
    let Some(index) = action
        .strip_prefix("toggle:")
        .and_then(|value| value.parse::<usize>().ok())
    else {
        return Vec::new();
    };
    toggle(state, index)
}

fn toggle(state: &mut Value, index: usize) -> Vec<DomEvent> {
    let Some(values) = state.get_mut("values").and_then(Value::as_array_mut) else {
        return Vec::new();
    };
    if index >= values.len() {
        return Vec::new();
    }
    let current =
        values[index].as_f64().unwrap_or(0.0) != 0.0 || values[index].as_bool() == Some(true);
    values[index] = Value::from(if current { 0 } else { 1 });
    vec![DomEvent::input("switch-change", Value::from(index))]
}

fn replace_group_contents(html: &str, anchor: &str, contents: &str) -> String {
    let Some(start) = html.find(anchor) else {
        return html.to_owned();
    };
    let content_start = start + anchor.len();
    let Some(relative_end) = html[content_start..].find("</g>") else {
        return html.to_owned();
    };
    let end = content_start + relative_end;
    let mut output = html.to_owned();
    output.replace_range(content_start..end, contents);
    output
}

#[allow(dead_code)]
fn _is_on(value: &Value) -> bool {
    bool_prop(value, "value", false)
}
