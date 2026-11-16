use serde_json::Value;

use crate::{generated, render::Rendered};

pub fn render(_state: &Value) -> Rendered {
    generated::render("wokwi-ssd1306").expect("SSD1306 default render")
}
