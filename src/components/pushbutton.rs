use serde_json::Value;

use crate::{
    events::DomEvent,
    render::Rendered,
    util::{bool_prop, escape_html, render_template, set_bool, string_prop},
};

const STYLES: &str = r#"
:host {
  display: inline-flex;
  flex-direction: column;
}

button {
  border: none;
  background: none;
  padding: 0;
  margin: 0;
  text-decoration: none;
  -webkit-appearance: none;
  -moz-appearance: none;
}

.button-active-circle {
  opacity: 0;
}

button:active .button-active-circle {
  opacity: 1;
}

.clickable-element {
  cursor: pointer;
}

.label {
  width: 0;
  min-width: 100%;
  font-size: 12px;
  text-align: center;
  color: gray;
  position: relative;
  line-height: 1;
  top: -2px;
}
"#;

const XRAY: &str = r#"
<rect
  style="opacity:0.3;fill:#999999;stroke-width:0.563001;paint-order:stroke markers fill"
  width="12.087865"
  height="1.0371729"
  x="-0.00075517414"
  y="2.9106798"
/>
<rect
  style="opacity:0.3;fill:#999999;stroke-width:0.534365;paint-order:stroke markers fill"
  width="12.087865"
  height="0.93434691"
  x="-0.071111664"
  y="8.0458994"
/>
"#;

pub fn render(state: &Value) -> Rendered {
    let color = escape_html(string_prop(state, "color", "red"));
    let label = escape_html(string_prop(state, "label", ""));
    let pressed = bool_prop(state, "pressed", false);
    let xray = if bool_prop(state, "xray", false) {
        XRAY
    } else {
        ""
    };
    let unique_id = "pushbutton";
    let button_fill = if pressed {
        format!("url(#grad-down-{unique_id})")
    } else {
        format!("url(#grad-up-{unique_id})")
    };
    let svg = render_template(
        include_str!("templates/pushbutton.svg"),
        &[
            ("uniqueId", unique_id.into()),
            ("color", color.clone()),
            ("buttonFill", button_fill),
            ("xray", xray.into()),
        ],
    );
    let aria_label = escape_html(&format!("{label} {color} pushbutton"));
    let html = format!(
        r#"<button aria-label="{aria_label}" data-on-pointerdown="down" data-on-mousedown="down" data-on-touchstart="down" data-on-pointerup="up" data-on-mouseup="up" data-on-touchend="up" data-on-pointerleave="leave" data-on-keydown="keydown" data-on-keyup="keyup">{svg}</button><span class="label">{label}</span>"#
    );

    Rendered {
        html,
        styles: STYLES.into(),
    }
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    match action {
        "down" => down(state),
        "up" => up(state, ctrl_cmd_pressed(payload)),
        "leave" => {
            if !bool_prop(state, "sticky", false) {
                up(state, ctrl_cmd_pressed(payload))
            } else {
                Vec::new()
            }
        }
        "keydown" => {
            if space_key(payload) {
                down(state)
            } else {
                Vec::new()
            }
        }
        "keyup" => {
            if space_key(payload) {
                up(state, ctrl_cmd_pressed(payload))
            } else {
                Vec::new()
            }
        }
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
    if !bool_prop(state, "pressed", false) {
        return Vec::new();
    }
    if ctrl_cmd {
        set_bool(state, "sticky", true);
        Vec::new()
    } else {
        set_bool(state, "sticky", false);
        set_bool(state, "pressed", false);
        vec![DomEvent::event("button-release")]
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
