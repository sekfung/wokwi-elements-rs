use std::sync::OnceLock;

use crate::{pins::PinInfo, render::Rendered};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultRender {
    tag: String,
    html: String,
    styles: String,
    pin_info: Option<Vec<PinInfo>>,
}

fn defaults() -> &'static Vec<DefaultRender> {
    static DEFAULTS: OnceLock<Vec<DefaultRender>> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        serde_json::from_str(include_str!("defaults.json"))
            .expect("generated component defaults must be valid")
    })
}

fn find(tag: &str) -> Option<&'static DefaultRender> {
    defaults().iter().find(|component| component.tag == tag)
}

pub fn render(tag: &str) -> Option<Rendered> {
    find(tag).map(|component| Rendered {
        html: component.html.clone(),
        styles: component.styles.clone(),
    })
}

pub fn pin_info(tag: &str) -> Option<Vec<PinInfo>> {
    find(tag)?.pin_info.clone()
}
