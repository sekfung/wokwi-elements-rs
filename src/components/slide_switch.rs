use serde_json::Value;

use crate::{
    events::DomEvent,
    render::Rendered,
    util::{bool_prop, number_prop, set_bool},
};

const STYLES: &str = r#"
.hide-input {
  position: absolute;
  clip: rect(0 0 0 0);
  width: 1px;
  height: 1px;
  margin: -1px;
}
svg #handle {
  transition: transform 0.2s linear;
}
input:checked + svg #handle {
  transform: translate(2px, 0);
}
input:focus + svg #handle {
  stroke-width: 0.4px;
  stroke: #8080ff;
}
"#;

pub fn render(state: &Value) -> Rendered {
    let value = number_prop(state, "value", 0.0);
    let checked = if value != 0.0 { " checked" } else { "" };
    Rendered {
        html: format!(
            r#"<input tabindex="0" type="checkbox" class="hide-input"{checked} data-on-input="change" />{}"#,
            include_str!("templates/slide-switch.svg")
        ),
        styles: STYLES.into(),
    }
}

pub fn action(state: &mut Value, action: &str, payload: &Value) -> Vec<DomEvent> {
    let next = match action {
        "toggle" => {
            if number_prop(state, "value", 0.0) != 0.0 {
                0
            } else {
                1
            }
        }
        "change" => {
            if bool_prop(payload, "checked", false) {
                1
            } else {
                0
            }
        }
        _ => return Vec::new(),
    };

    if let Some(object) = state.as_object_mut() {
        object.insert("value".into(), Value::from(next));
    }
    let _ = set_bool;
    vec![DomEvent::input("input", Value::from(next))]
}
