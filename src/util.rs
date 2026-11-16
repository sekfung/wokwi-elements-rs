use serde_json::Value;

pub fn bool_prop(state: &Value, name: &str, default: bool) -> bool {
    state.get(name).and_then(Value::as_bool).unwrap_or(default)
}

pub fn set_bool(state: &mut Value, name: &str, value: bool) {
    if let Some(object) = state.as_object_mut() {
        object.insert(name.into(), Value::Bool(value));
    }
}

pub fn number_prop(state: &Value, name: &str, default: f64) -> f64 {
    state.get(name).and_then(Value::as_f64).unwrap_or(default)
}

pub fn string_prop<'a>(state: &'a Value, name: &str, default: &'a str) -> &'a str {
    state.get(name).and_then(Value::as_str).unwrap_or(default)
}

pub fn numberish_prop(state: &Value, name: &str, default: f64) -> f64 {
    match state.get(name) {
        Some(Value::Number(value)) => value.as_f64().unwrap_or(default),
        Some(Value::String(value)) => value.parse().unwrap_or(default),
        _ => default,
    }
}

pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub fn render_template(template: &str, values: &[(&str, String)]) -> String {
    let mut rendered = template.to_owned();
    for (key, value) in values {
        rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
    }
    rendered
}

pub fn format_js_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    if value.fract() == 0.0 && value.abs() < 1e21 {
        return format!("{value:.0}");
    }
    value.to_string()
}
