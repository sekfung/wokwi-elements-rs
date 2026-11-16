use serde_json::Value;

use crate::{generated, render::Rendered, util::bool_prop};

pub fn render(state: &Value) -> Rendered {
    let base = generated::render("wokwi-ili9341").expect("ILI9341 default render");
    let h = bool_prop(state, "flipHorizontal", false);
    let v = bool_prop(state, "flipVertical", false);
    let style = if h || v {
        format!(
            "transform: scaleX({}) scaleY({});",
            if h { -1 } else { 1 },
            if v { -1 } else { 1 }
        )
    } else {
        String::new()
    };
    let html = base.html.replace(
        r##"<canvas class="pixelated" width="240" height="320" style="">"##,
        &format!(r##"<canvas class="pixelated" width="240" height="320" style="{style}">"##),
    );
    Rendered {
        html,
        styles: base.styles,
    }
}
