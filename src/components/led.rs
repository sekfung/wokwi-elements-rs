use serde_json::Value;

use crate::{
    pins::PinInfo,
    render::Rendered,
    util::{bool_prop, escape_html, number_prop, render_template, string_prop},
};

const STYLES: &str = r#"
:host {
  display: inline-block;
}

.led-container {
  display: flex;
  flex-direction: column;
  width: 40px;
}

.led-label {
  font-size: 10px;
  text-align: center;
  color: gray;
  position: relative;
  line-height: 1;
  top: -8px;
}
"#;

pub fn render(state: &Value) -> Rendered {
    let value = bool_prop(state, "value", false);
    let brightness = number_prop(state, "brightness", 1.0);
    let color = string_prop(state, "color", "red");
    let light_color = state
        .get("lightColor")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| match color.to_ascii_lowercase().as_str() {
            "red" => "#ff8080".to_owned(),
            "green" => "#80ff80".to_owned(),
            "blue" => "#8080ff".to_owned(),
            "yellow" => "#ffff80".to_owned(),
            "orange" => "#ffcf80".to_owned(),
            "white" => "#ffffff".to_owned(),
            "purple" => "#ff80ff".to_owned(),
            _ => color.to_owned(),
        });
    let opacity = if brightness == 0.0 {
        0.0
    } else {
        0.3 + brightness * 0.7
    };
    let display = if value && brightness > f64::EPSILON {
        ""
    } else {
        "none"
    };
    let x_scale = if bool_prop(state, "flip", false) {
        "-1"
    } else {
        "1"
    };
    let label = escape_html(string_prop(state, "label", ""));
    let svg = render_template(
        include_str!("templates/led.svg"),
        &[
            ("xScale", x_scale.into()),
            ("color", escape_html(color)),
            ("display", display.into()),
            ("lightColor", escape_html(&light_color)),
            ("opacity", opacity.to_string()),
        ],
    );

    Rendered {
        html: format!(
            r#"<div class="led-container">{svg}<span class="led-label">{label}</span></div>"#
        ),
        styles: STYLES.into(),
    }
}

pub fn pin_info(state: &Value) -> Vec<PinInfo> {
    let flip = bool_prop(state, "flip", false);
    let anode_x = if flip { 15.0 } else { 25.0 };
    let cathode_x = if flip { 25.0 } else { 15.0 };

    vec![
        PinInfo {
            name: "A".into(),
            x: anode_x,
            y: 42.0,
            number: None,
            description: Some("Anode".into()),
            signals: vec![],
            no_breadboard: None,
        },
        PinInfo {
            name: "C".into(),
            x: cathode_x,
            y: 42.0,
            number: None,
            description: Some("Cathode".into()),
            signals: vec![],
            no_breadboard: None,
        },
    ]
}
