use serde_json::Value;

use crate::{generated, render::Rendered, util::bool_prop};

pub fn render(state: &Value) -> Rendered {
    let base = generated::render("wokwi-buzzer").expect("buzzer default render");
    let html = if bool_prop(state, "hasSignal", false) {
        base.html.replace("visibility: hidden", "visibility: ")
    } else {
        base.html
    };
    Rendered {
        html,
        styles: base.styles,
    }
}
