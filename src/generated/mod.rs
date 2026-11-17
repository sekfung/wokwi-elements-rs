#[cfg(not(feature = "native-pack"))]
use std::sync::OnceLock;

use crate::{pins::PinInfo, render::Rendered};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DefaultRender {
    #[cfg(not(feature = "native-pack"))]
    tag: String,
    html: String,
    styles: String,
    pin_info: Option<Vec<PinInfo>>,
}

#[cfg(not(feature = "native-pack"))]
fn defaults() -> &'static Vec<DefaultRender> {
    static DEFAULTS: OnceLock<Vec<DefaultRender>> = OnceLock::new();
    DEFAULTS.get_or_init(|| {
        serde_json::from_str(include_str!("defaults.json"))
            .expect("generated component defaults must be valid")
    })
}

#[cfg(feature = "native-pack")]
include!(concat!(env!("OUT_DIR"), "/native-defaults.rs"));

fn with_default<T>(tag: &str, map: impl FnOnce(&DefaultRender) -> T) -> Option<T> {
    #[cfg(feature = "native-pack")]
    {
        let component = serde_json::from_str(native_default(tag)?).ok()?;
        Some(map(&component))
    }
    #[cfg(not(feature = "native-pack"))]
    {
        defaults()
            .iter()
            .find(|component| component.tag == tag)
            .map(map)
    }
}

pub fn render(tag: &str) -> Option<Rendered> {
    with_default(tag, |component| Rendered {
        html: component.html.clone(),
        styles: component.styles.clone(),
    })
}

pub fn pin_info(tag: &str) -> Option<Vec<PinInfo>> {
    with_default(tag, |component| component.pin_info.clone()).flatten()
}
