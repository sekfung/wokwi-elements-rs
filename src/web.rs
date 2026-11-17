use serde_json::{json, Value};
#[cfg(not(feature = "native-pack"))]
use wasm_bindgen::prelude::*;

use crate::{contract, events, pin_info, render};

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn install_debug_hook() {
    #[cfg(all(
        feature = "debug-hook",
        not(feature = "native-pack"),
        target_arch = "wasm32"
    ))]
    console_error_panic_hook::set_once();
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn component_contracts() -> String {
    contract::contracts_json().to_owned()
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn default_state_json(tag: &str) -> String {
    contract::default_state(tag)
        .unwrap_or_else(|| json!({}))
        .to_string()
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn render_component(tag: &str, state_json: &str) -> String {
    let state: serde_json::Value = serde_json::from_str(state_json).unwrap_or(Value::Null);
    match render(tag, &state) {
        Some(rendered) => json!({
            "html": rendered.html,
            "styles": rendered.styles,
        })
        .to_string(),
        None => "null".to_owned(),
    }
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn pin_info_json(tag: &str, state_json: &str) -> String {
    let state: serde_json::Value = serde_json::from_str(state_json).unwrap_or(Value::Null);
    pin_info(tag, &state)
        .map(|pins| serde_json::to_string(&pins).unwrap_or_else(|_| "[]".to_owned()))
        .unwrap_or_else(|| "null".to_owned())
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
#[cfg_attr(feature = "native-pack", allow(dead_code))]
pub fn dispatch_action(tag: &str, state_json: &str, action: &str, payload_json: &str) -> String {
    let mut state: serde_json::Value =
        serde_json::from_str(state_json).unwrap_or_else(|_| json!({}));
    let payload: serde_json::Value =
        serde_json::from_str(payload_json).unwrap_or_else(|_| json!({}));
    events::dispatch(tag, &mut state, action, &payload)
        .and_then(|outcome| serde_json::to_string(&outcome).ok())
        .unwrap_or_else(|| "null".into())
}

#[cfg_attr(not(feature = "native-pack"), wasm_bindgen)]
pub fn invoke_method(tag: &str, state_json: &str, method: &str, args_json: &str) -> String {
    let mut state: serde_json::Value =
        serde_json::from_str(state_json).unwrap_or_else(|_| json!({}));
    let args: serde_json::Value = serde_json::from_str(args_json).unwrap_or_else(|_| json!([]));
    if events::invoke(tag, &mut state, method, &args) {
        state.to_string()
    } else {
        "null".into()
    }
}
