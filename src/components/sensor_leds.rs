use serde_json::Value;

use crate::{generated, render::Rendered, util::bool_prop};

pub fn render(tag: &str, state: &Value) -> Rendered {
    let base = generated::render(tag).expect("sensor default render");
    let mut html = base.html;

    match tag {
        "wokwi-gas-sensor" => {
            if bool_prop(state, "ledD0", false) {
                html = insert_after_false(
                    &html,
                    "id=\"rect18-0\"",
                    r##"<circle cx="85" cy="50" r="1.8" fill="#03f704" filter="url(#ledFilter)"></circle>"##,
                );
            }
            if bool_prop(state, "ledPower", false) {
                html = insert_after_false(
                    &html,
                    "id=\"rect18\"",
                    r##"<circle cx="85.5" cy="8" r="1.8" fill="#03f704" filter="url(#ledFilter)"></circle>"##,
                );
            }
        }
        "wokwi-flame-sensor" => {
            if bool_prop(state, "ledSignal", false) {
                html = insert_after_false(
                    &html,
                    r##"<rect x="149" y="48.1""##,
                    r##"<circle cx="153.2" cy="51.3" r="7" fill="#80ff80" filter="url(#ledFilter)"></circle>"##,
                );
            }
            if bool_prop(state, "ledPower", false) {
                html = insert_after_false(
                    &html,
                    "<filter id=\"ledFilter\"",
                    r##"<circle cx="153.2" cy="7.5" r="7" fill="#80ff80" filter="url(#ledFilter)"></circle>"##,
                );
            }
        }
        "wokwi-big-sound-sensor" | "wokwi-small-sound-sensor" => {
            if bool_prop(state, "led1", false) || bool_prop(state, "ledSignal", false) {
                let radius = if tag == "wokwi-big-sound-sensor" {
                    5
                } else {
                    7
                };
                html = insert_after_false(
                    &html,
                    r##"<rect x="54.3" y="45.1""##,
                    &format!(
                        r##"<circle cx="56" cy="47" r="{radius}" fill="#80ff80" filter="url(#ledFilter)"></circle>"##
                    ),
                );
            }
            if bool_prop(state, "led2", false) || bool_prop(state, "ledPower", false) {
                let radius = if tag == "wokwi-big-sound-sensor" {
                    5
                } else {
                    7
                };
                html = insert_after_false(
                    &html,
                    "<filter id=\"ledFilter\"",
                    &format!(
                        r##"<circle cx="56" cy="4" r="{radius}" fill="#80ff80" filter="url(#ledFilter)"></circle>"##
                    ),
                );
            }
        }
        "wokwi-photoresistor-sensor" => {
            if bool_prop(state, "ledDO", false) {
                html = insert_after_false(
                    &html,
                    r##"<rect x="121" y="52.5""##,
                    r##"<circle cx="124.5" cy="55" r="4" fill="red" filter="url(#ledFilter)"></circle>"##,
                );
            }
            if bool_prop(state, "ledPower", false) {
                html = insert_after_false(
                    &html,
                    "<filter id=\"ledFilter\"",
                    r##"<circle cx="124.5" cy="7" r="4" fill="green" filter="url(#ledFilter)"></circle>"##,
                );
            }
        }
        "wokwi-mpu6050" => {
            if bool_prop(state, "led1", false) {
                html = insert_after_false(
                    &html,
                    "<filter id=\"ledFilter\"",
                    r##"<circle cx="13.9" cy="25.5" r="3.5" fill="#80ff80" filter="url(#ledFilter)"></circle>"##,
                );
            }
        }
        _ => {}
    }

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
