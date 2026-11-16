use serde_json::Value;

use crate::{
    generated,
    render::Rendered,
    util::{escape_html, format_js_number, numberish_prop, string_prop},
};

pub fn render(state: &Value) -> Rendered {
    let hands: Value = serde_json::from_str(include_str!("biaxial_hands.json"))
        .expect("generated biaxial hand templates must be valid");
    let outer_length = numberish_prop(state, "outerHandLength", 30.0).clamp(20.0, 70.0);
    let inner_length = numberish_prop(state, "innerHandLength", 30.0).clamp(20.0, 70.0);
    let outer_angle = numberish_prop(state, "outerHandAngle", 0.0) + 270.0;
    let inner_angle = numberish_prop(state, "innerHandAngle", 0.0) + 270.0;
    let outer_color = escape_html(string_prop(state, "outerHandColor", "gold"));
    let inner_color = escape_html(string_prop(state, "innerHandColor", "silver"));
    let outer_shape = string_prop(state, "outerHandShape", "plain");
    let inner_shape = string_prop(state, "innerHandShape", "plain");
    let (outer_path, outer_x, outer_y) = hand(&hands, "outer", outer_shape, outer_length);
    let (inner_path, inner_x, inner_y) = hand(&hands, "inner", inner_shape, inner_length);
    let outer_fragment = format!(
        r##"<path class="cls-h" transform="translate(60 {}) rotate({}) translate(-{}, -{})" fill="{}" d="{}"></path>"##,
        format_js_number(77.0 + outer_y),
        format_js_number(outer_angle),
        format_js_number(outer_x),
        format_js_number(outer_y),
        outer_color,
        outer_path
    );
    let inner_fragment = format!(
        r##"<path class="cls-h" transform="translate(60 {}) rotate({}) translate(-{}, -{})" fill="{}" d="{}"></path>"##,
        format_js_number(77.0 + inner_y),
        format_js_number(inner_angle),
        format_js_number(inner_x),
        format_js_number(inner_y),
        inner_color,
        inner_path
    );
    let base = generated::render("wokwi-biaxial-stepper").expect("biaxial default render");
    let html = replace_hand(
        replace_hand(base.html, 1, &inner_fragment),
        0,
        &outer_fragment,
    );
    Rendered {
        html,
        styles: base.styles,
    }
}

fn hand(hands: &Value, prefix: &str, shape: &str, length: f64) -> (String, f64, f64) {
    let mut capitalized = shape.to_owned();
    if let Some(first) = capitalized.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    let key = format!("{prefix}{capitalized}Hand");
    let descriptor = hands
        .get(&key)
        .or_else(|| hands.get(&format!("{prefix}PlainHand")))
        .expect("plain hand");
    let path = descriptor
        .get("pathTemplate")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .replace("{len}", &format_js_number(length));
    let x = descriptor
        .get("xOff")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let y = descriptor
        .get("yOff")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    (path, x, y)
}

fn replace_hand(html: String, index: usize, fragment: &str) -> String {
    let marker = r##"<path class="cls-h""##;
    let mut search_from = 0usize;
    for occurrence in 0..=index {
        let Some(start) = html[search_from..]
            .find(marker)
            .map(|offset| search_from + offset)
        else {
            return html;
        };
        let Some(end_offset) = html[start..].find("</path>") else {
            return html;
        };
        let end = start + end_offset + "</path>".len();
        if occurrence == index {
            let mut output = String::with_capacity(html.len() + fragment.len());
            output.push_str(&html[..start]);
            output.push_str(fragment);
            output.push_str(&html[end..]);
            return output;
        }
        search_from = end;
    }
    html
}
