use serde_json::Value;

use crate::{
    events::DomEvent,
    generated,
    render::Rendered,
    util::{bool_prop, escape_html, set_bool, string_prop},
};

const XRAY: &str = r##"
<rect style="opacity:0.3;fill:#b3b3b3;stroke-width:3.86235;paint-order:stroke markers fill" width="12.124171" height="0.51113945" x="-0.047361366" y="0.90351838" rx="0.12476496"></rect>
<rect style="opacity:0.3;fill:#b3b3b3;stroke-width:3.86235;paint-order:stroke markers fill" width="12.124171" height="0.51113945" x="-0.098103404" y="10.614529" rx="0.12476496"></rect>
"##;

pub fn render(state: &Value) -> Rendered {
    let color = escape_html(string_prop(state, "color", "red"));
    let label = escape_html(string_prop(state, "label", ""));
    let pressed = bool_prop(state, "pressed", false);
    let base = generated::render("wokwi-pushbutton-6mm").expect("6mm button default render");
    let mut html = base
        .html
        .replace("stop-color=\"red\"", &format!("stop-color=\"{color}\""))
        .replace("fill=\"red\"", &format!("fill=\"{color}\""));

    if pressed {
        html = html.replace(
            "fill=\"url(#grad-up-pushbutton0)\"",
            "fill=\"url(#grad-down-pushbutton0)\"",
        );
    }
    html = html.replace(
        "<button aria-label=\" red pushbutton 6mm\">",
        &format!(
            "<button aria-label=\"{label} {color} pushbutton 6mm\" data-on-pointerdown=\"down\" data-on-mousedown=\"down\" data-on-touchstart=\"down\" data-on-pointerup=\"up\" data-on-mouseup=\"up\" data-on-touchend=\"up\" data-on-pointerleave=\"up\" data-on-keydown=\"keydown\" data-on-keyup=\"keyup\">"
        ),
    );
    html = html.replace(
        "<span class=\"label\"></span>",
        &format!("<span class=\"label\">{label}</span>"),
    );

    if bool_prop(state, "xray", false) {
        let anchor = "id=\"rect18-0-4\" width=\"1.3927531\" height=\"0.85429525\" x=\"11.985411\" y=\"10.744086\" rx=\"0.014332262\"></rect>";
        html = html.replacen(anchor, &format!("{anchor}{XRAY}"), 1);
    }

    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    match action {
        "down" => down(state),
        "up" => up(state, ctrl_cmd_pressed(payload)),
        "keydown" if space_key(payload) => down(state),
        "keyup" if space_key(payload) => up(state, ctrl_cmd_pressed(payload)),
        _ => Vec::new(),
    }
}

fn down(state: &mut Value) -> Vec<DomEvent> {
    if bool_prop(state, "pressed", false) {
        return Vec::new();
    }
    set_bool(state, "pressed", true);
    vec![DomEvent::event("button-press")]
}

fn up(state: &mut Value, ctrl_cmd: bool) -> Vec<DomEvent> {
    if bool_prop(state, "pressed", false) && !ctrl_cmd {
        set_bool(state, "pressed", false);
        vec![DomEvent::event("button-release")]
    } else {
        Vec::new()
    }
}

fn space_key(payload: &Value) -> bool {
    matches!(
        payload.get("key").and_then(Value::as_str),
        Some(" ") | Some("Spacebar")
    )
}

fn ctrl_cmd_pressed(payload: &Value) -> bool {
    bool_prop(payload, "ctrlKey", false) || bool_prop(payload, "metaKey", false)
}
