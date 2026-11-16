use serde_json::Value;

use crate::{
    events::DomEvent,
    generated,
    render::Rendered,
    util::{bool_prop, set_bool},
};

pub fn render(tag: &str, state: &Value) -> Rendered {
    let base = generated::render(tag).expect("board default render");
    let mut html = base.html;

    let leds: &[(&str, &str, &str)] = match tag {
        "wokwi-arduino-uno" => &[
            (
                "ledPower",
                "translate(57.3, 16.21)",
                led(1.3, 0.55, "#80ff80"),
            ),
            ("led13", "translate(26.87,11.69)", led(1.3, 0.55, "#ff8080")),
            ("ledTX", "translate(26.9, 16.2)", led(0.975, 0.55, "yellow")),
            ("ledRX", "translate(26.9, 18.5)", led(0.975, 0.55, "yellow")),
        ],
        "wokwi-arduino-nano" => &[
            ("ledTX", "translate(27.7 5)", led(1.3, 0.55, "#ff8080")),
            ("ledRX", "translate(27.7 7)", led(1.3, 0.55, "#80ff80")),
            ("ledPower", "translate(27.7 9)", led(1.3, 0.55, "#80ff80")),
            ("led13", "translate(27.7 11)", led(1.3, 0.55, "#ffff80")),
        ],
        "wokwi-arduino-mega" => &[
            (
                "ledPower",
                "translate(72.20 15.58)",
                led(1.3, 0.55, "#80ff80"),
            ),
            ("led13", "translate(26 13.86)", led(1.3, 0.55, "#ff8080")),
            ("ledTX", "translate(26 18.52)", led(0.975, 0.55, "yellow")),
            ("ledRX", "translate(26 20.75)", led(0.975, 0.55, "yellow")),
        ],
        "wokwi-esp32-devkit-v1" => &[
            ("ledPower", "x=\"35\" y=\"111\"", led(37.0, 112.5, "red")),
            ("led1", "x=\"69.5\" y=\"110\"", led(71.5, 112.5, "blue")),
        ],
        "wokwi-franzininho" => &[
            (
                "ledPower",
                "cx=\"33.6\" cy=\"25.9\" r=\"1.2\"",
                led(33.6, 25.9, "#03f704"),
            ),
            (
                "led1",
                "cx=\"38.35\" cy=\"25.9\" r=\"1.2\"",
                led(38.35, 25.9, "#fcfd00"),
            ),
        ],
        _ => &[],
    };

    for (property, anchor, fragment) in leds {
        if bool_prop(state, property, false) {
            html = insert_after_false(&html, anchor, fragment);
        }
    }

    if matches!(
        tag,
        "wokwi-arduino-uno" | "wokwi-arduino-nano" | "wokwi-arduino-mega" | "wokwi-franzininho"
    ) {
        html = add_reset_attrs(&html, bool_prop(state, "resetPressed", false));
    }

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, event: &str, payload: &Value) -> Vec<DomEvent> {
    match event {
        "down" => {
            if bool_prop(state, "resetPressed", false) {
                Vec::new()
            } else {
                set_bool(state, "resetPressed", true);
                vec![DomEvent::custom_detail(
                    "button-press",
                    Value::String("reset".into()),
                )]
            }
        }
        "up" | "leave" => {
            if bool_prop(state, "resetPressed", false) {
                set_bool(state, "resetPressed", false);
                vec![DomEvent::custom_detail(
                    "button-release",
                    Value::String("reset".into()),
                )]
            } else {
                Vec::new()
            }
        }
        "keydown" if space_key(payload) => action(state, "down", payload),
        "keyup" if space_key(payload) => action(state, "up", payload),
        _ => Vec::new(),
    }
}

fn led(cx: f64, cy: f64, color: &str) -> &'static str {
    match (cx, cy, color) {
        (1.3, 0.55, "#80ff80") => "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\" fill=\"#80ff80\" filter=\"url(#ledFilter)\"></circle>",
        (1.3, 0.55, "#ff8080") => "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\" fill=\"#ff8080\" filter=\"url(#ledFilter)\"></circle>",
        (1.3, 0.55, "#ffff80") => "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\" fill=\"#ffff80\" filter=\"url(#ledFilter)\"></circle>",
        (1.3, 0.55, "yellow") => "<circle cx=\"0.975\" cy=\"0.55\" r=\"1.3\" fill=\"yellow\" filter=\"url(#ledFilter)\"></circle>",
        (37.0, 112.5, "red") => "<circle cx=\"37\" cy=\"112.5\" r=\"4\" fill=\"red\" filter=\"url(#ledFilter)\"></circle>",
        (71.5, 112.5, "blue") => "<circle cx=\"71.5\" cy=\"112.5\" r=\"4\" fill=\"blue\" filter=\"url(#ledFilter)\"></circle>",
        (33.6, 25.9, "#03f704") => "<circle cx=\"33.6\" cy=\"25.9\" r=\"1.8\" fill=\"#03f704\" filter=\"url(#ledFilter)\"></circle>",
        (38.35, 25.9, "#fcfd00") => "<circle cx=\"38.35\" cy=\"25.9\" r=\"1.8\" fill=\"#fcfd00\" filter=\"url(#ledFilter)\"></circle>",
        _ => "",
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

fn add_reset_attrs(html: &str, pressed: bool) -> String {
    let Some(anchor_index) = html.find("id=\"reset-button\"") else {
        return html.to_owned();
    };
    let Some(relative_end) = html[anchor_index..].find('>') else {
        return html.to_owned();
    };
    let end = anchor_index + relative_end;
    let style = if pressed {
        " style=\"stroke: #333\""
    } else {
        ""
    };
    let mut output = html.to_owned();
    output.insert_str(
        end,
        &format!(
            "{style} data-on-mousedown=\"down\" data-on-touchstart=\"down\" data-on-mouseup=\"up\" data-on-mouseleave=\"leave\" data-on-touchend=\"leave\" data-on-keydown=\"keydown\" data-on-keyup=\"keyup\""
        ),
    );
    output
}

fn space_key(payload: &Value) -> bool {
    matches!(
        payload.get("key").and_then(Value::as_str),
        Some(" ") | Some("Spacebar")
    )
}
