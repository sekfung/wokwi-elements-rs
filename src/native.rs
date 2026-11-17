//! Import-free artwork guest for the native Boardless runtime.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cell::RefCell;

const MAX_REQUEST: usize = 64 * 1024;
const MAX_RESPONSE: usize = 4 * 1024 * 1024;
const CATALOG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/native-catalog.json"));

thread_local! {
    static BUFFERS: RefCell<Buffers> = const { RefCell::new(Buffers {
        input: Vec::new(),
        output: Vec::new(),
    }) };
}

struct Buffers {
    input: Vec<u8>,
    output: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Request {
    Catalog,
    Render {
        tag: String,
        state: Map<String, Value>,
    },
}

#[derive(Serialize)]
struct Artwork {
    html: String,
    styles: String,
    pins: Vec<crate::PinInfo>,
}

#[no_mangle]
pub extern "C" fn element_abi_version() -> u32 {
    1
}

/// The host may write exactly `len` bytes at the returned offset.
#[no_mangle]
pub extern "C" fn element_input(len: u32) -> u32 {
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        buffers.output.clear();
        buffers.input.clear();
        let len = len as usize;
        if len == 0 || len > MAX_REQUEST {
            return 0;
        }
        buffers.input.resize(len, 0);
        buffers.input.as_mut_ptr() as u32
    })
}

/// The host must copy the return buffer before invoking another operation.
#[no_mangle]
pub extern "C" fn element_call() -> u64 {
    BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let response = respond(&buffers.input);
        buffers.input.clear();
        buffers.output.clear();
        let Some(response) = response.filter(|bytes| bytes.len() <= MAX_RESPONSE) else {
            return 0;
        };
        buffers.output = response;
        let pointer = buffers.output.as_ptr() as u32;
        (u64::from(pointer) << 32) | buffers.output.len() as u64
    })
}

fn respond(input: &[u8]) -> Option<Vec<u8>> {
    match serde_json::from_slice(input).ok()? {
        Request::Catalog => Some(CATALOG.to_vec()),
        Request::Render { tag, state } => {
            let state = Value::Object(state);
            let rendered = crate::render(&tag, &state)?;
            serde_json::to_vec(&Artwork {
                html: rendered.html,
                styles: rendered.styles,
                pins: crate::pin_info(&tag, &state).unwrap_or_default(),
            })
            .ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn catalog_and_render_match_the_existing_element_api() {
        let result = respond(br#"{"op":"catalog"}"#).expect("catalog");
        let catalog: Vec<Value> = serde_json::from_slice(&result).unwrap();
        assert_eq!(catalog.len(), crate::tag_names().len());
        for (entry, tag) in catalog.iter().zip(crate::tag_names()) {
            assert_eq!(entry["tag"], *tag);
            let expected_state =
                serde_json::from_str::<Value>(&crate::default_state_json(tag)).unwrap();
            assert_eq!(entry["default_state"], expected_state, "{tag}");
            let request = json!({"op":"render", "tag":tag, "state":entry["default_state"]});
            let reply = respond(&serde_json::to_vec(&request).unwrap()).expect(tag);
            let rendered: Value = serde_json::from_slice(&reply).unwrap();
            let expected = crate::render(tag, &entry["default_state"]).unwrap();
            assert_eq!(rendered["html"], expected.html, "{tag}");
            assert_eq!(rendered["styles"], expected.styles, "{tag}");
            let expected_pins = serde_json::from_str::<Option<Vec<Value>>>(&crate::pin_info_json(
                tag,
                &entry["default_state"].to_string(),
            ))
            .unwrap()
            .unwrap_or_default();
            assert_eq!(rendered["pins"], Value::Array(expected_pins), "{tag}");
        }
    }

    #[test]
    fn rejects_unknown_operations_elements_and_invalid_state() {
        for request in [
            br#"{"op":"unknown"}"#.as_slice(),
            br#"{"op":"render","tag":"unregistered","state":{}}"#,
            br#"{"op":"render","tag":"wokwi-led","state":null}"#,
        ] {
            assert!(respond(request).is_none());
        }
    }

    #[test]
    fn rejects_oversized_input_and_discards_previous_request() {
        assert_eq!(element_abi_version(), 1);
        BUFFERS.with(|buffers| buffers.borrow_mut().input = br#"{"op":"catalog"}"#.to_vec());
        assert_ne!(element_call() as u32, 0);
        assert_eq!(element_call(), 0);
        assert_eq!(element_input(MAX_REQUEST as u32 + 1), 0);
        assert_eq!(element_call(), 0);
    }
}
