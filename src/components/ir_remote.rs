use serde_json::{json, Value};

use crate::{events::DomEvent, generated, render::Rendered};

pub fn render(state: &Value) -> Rendered {
    let base = generated::render("wokwi-ir-remote").expect("IR remote default render");
    let mut html = base.html.replacen(
        "<svg version=\"1.1\"",
        "<svg data-on-mousedown=\"down\" data-on-mouseup=\"up\" data-on-touchstart=\"down\" data-on-touchend=\"up\" data-on-keydown=\"keydown\" data-on-keyup=\"keyup\" version=\"1.1\"",
        1,
    );
    if let Some(active) = state.get("_activeKey").and_then(Value::as_str) {
        html = set_active_class(&html, active);
    }
    Rendered {
        html,
        styles: base.styles,
    }
}

pub fn action(state: &mut Value, event: &str, payload: &Value) -> Vec<DomEvent> {
    let key = payload
        .get("dataBtn")
        .and_then(Value::as_str)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            payload
                .get("key")
                .and_then(Value::as_str)
                .and_then(keyboard_key)
                .map(str::to_owned)
        });
    let Some(key) = key else { return Vec::new() };
    let Some(code) = ir_code(&key) else {
        return Vec::new();
    };
    if let Some(object) = state.as_object_mut() {
        object.insert(
            "_activeKey".into(),
            if event == "down" || event == "keydown" {
                Value::String(key.clone())
            } else {
                Value::Null
            },
        );
    }
    let detail = json!({ "key": key, "irCode": code });
    match event {
        "down" | "keydown" => vec![DomEvent::custom_detail("button-press", detail)],
        "up" | "keyup" => vec![DomEvent::custom_detail("button-release", detail)],
        _ => Vec::new(),
    }
}

fn set_active_class(html: &str, key: &str) -> String {
    let marker = format!("data-btn=\"{key}\"");
    let Some(marker_index) = html.find(&marker) else {
        return html.to_owned();
    };
    let Some(start) = html[..marker_index].rfind('<') else {
        return html.to_owned();
    };
    let Some(end_offset) = html[marker_index..].find('>') else {
        return html.to_owned();
    };
    let end = marker_index + end_offset;
    let tag = &html[start..end];
    let class_value = if tag.contains("class=\"red") {
        "red active"
    } else if tag.contains("class=\"black") {
        "black active"
    } else {
        "active"
    };
    let new_tag = if let Some(class_index) = tag.find("class=\"") {
        let value_start = start + class_index + 7;
        let value_end = html[value_start..end]
            .find('"')
            .map(|o| value_start + o)
            .unwrap_or(value_start);
        format!(
            "{}{}{}",
            &html[start..value_start],
            class_value,
            &html[value_end..end]
        )
    } else {
        format!("{tag} class=\"{class_value}\"")
    };
    format!("{}{}{}", &html[..start], new_tag, &html[end..])
}

fn keyboard_key(key: &str) -> Option<&'static str> {
    Some(match key.to_ascii_lowercase().as_str() {
        "o" => "power",
        "m" => "menu",
        "t" => "test",
        "+" => "plus",
        "b" => "back",
        "arrowleft" => "prev",
        "p" => "play",
        "arrowright" => "next",
        "0" => "0",
        "-" => "minus",
        "c" => "c",
        "1" => "1",
        "2" => "2",
        "3" => "3",
        "4" => "4",
        "5" => "5",
        "6" => "6",
        "7" => "7",
        "8" => "8",
        "9" => "9",
        _ => return None,
    })
}

fn ir_code(key: &str) -> Option<u32> {
    Some(match key {
        "power" => 0xa2,
        "menu" => 0xe2,
        "test" => 0x22,
        "plus" => 0x02,
        "back" => 0xc2,
        "prev" => 0xe0,
        "play" => 0xa8,
        "next" => 0x90,
        "0" => 0x68,
        "minus" => 0x98,
        "c" => 0xb0,
        "1" => 0x30,
        "2" => 0x18,
        "3" => 0x7a,
        "4" => 0x10,
        "5" => 0x38,
        "6" => 0x5a,
        "7" => 0x42,
        "8" => 0x4a,
        "9" => 0x52,
        _ => return None,
    })
}
