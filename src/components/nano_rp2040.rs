use serde_json::Value;

use crate::{
    generated,
    render::Rendered,
    util::{bool_prop, format_js_number, numberish_prop},
};

pub fn render(state: &Value) -> Rendered {
    let red = numberish_prop(state, "ledRed", 0.0);
    let green = numberish_prop(state, "ledGreen", 0.0);
    let blue = numberish_prop(state, "ledBlue", 0.0);
    let brightness = red.max(green).max(blue);
    let opacity = if brightness != 0.0 {
        0.3 + brightness * 0.7
    } else {
        0.0
    };
    let base = generated::render("wokwi-nano-rp2040-connect").expect("Nano RP2040 default render");
    let mut html = base.html;
    if bool_prop(state, "ledBuiltIn", false) {
        html = insert_after_false(
            &html,
            "<filter id=\"ledFilter\"",
            "<circle cx=\"14.5\" cy=\"14.5\" r=\"3\" fill=\"red\" filter=\"url(#ledFilter)\"></circle>",
        );
    }
    if bool_prop(state, "ledPower", false) {
        html = insert_after_false(
            &html,
            r##"<rect x="11.9" y="50.9""##,
            "<circle cx=\"14.5\" cy=\"53\" r=\"3\" fill=\"#80ff80\" filter=\"url(#ledFilter)\"></circle>",
        );
    }
    html = html.replace(
        r##"<circle cx="32.4" cy="25.4" r="3" filter="url(#ledFilter)" fill="rgb(0, 0, 0)" opacity="0"></circle>"##,
        &format!(
            r##"<circle cx="32.4" cy="25.4" r="3" filter="url(#ledFilter)" fill="rgb({}, {}, {})" opacity="{}"></circle>"##,
            format_js_number(red * 255.0),
            format_js_number(green * 255.0),
            format_js_number(blue * 255.0),
            format_js_number(opacity)
        ),
    );
    Rendered {
        html,
        styles: base.styles,
    }
}

fn insert_after_false(html: &str, anchor: &str, fragment: &str) -> String {
    let Some(anchor_index) = html.find(anchor) else {
        return html.to_owned();
    };
    let Some(relative_index) = html[anchor_index..].find("false") else {
        return html.to_owned();
    };
    let start = anchor_index + relative_index;
    let mut output = html.to_owned();
    output.replace_range(start..start + "false".len(), fragment);
    output
}
