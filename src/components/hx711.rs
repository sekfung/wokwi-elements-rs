use serde_json::Value;

use crate::{render::Rendered, util::string_prop};

pub fn render(state: &Value) -> Rendered {
    let variants: Value = serde_json::from_str(include_str!("../generated/hx711_variants.json"))
        .expect("generated HX711 variants must be valid");
    let requested = string_prop(state, "type", "50kg");
    let variant = variants
        .get(requested)
        .or_else(|| variants.get("50kg"))
        .expect("HX711 50kg variant");
    Rendered {
        html: variant
            .get("html")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        styles: variant
            .get("styles")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}
