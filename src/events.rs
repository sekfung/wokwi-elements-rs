use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomEvent {
    pub name: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Value>,
}

impl DomEvent {
    pub fn custom_detail(name: &str, detail: Value) -> Self {
        Self {
            name: name.into(),
            event_type: "CustomEvent".into(),
            detail: Some(detail),
        }
    }
}

impl DomEvent {
    pub fn event(name: &str) -> Self {
        Self {
            name: name.into(),
            event_type: "Event".into(),
            detail: None,
        }
    }

    pub fn custom(name: &str) -> Self {
        Self {
            name: name.into(),
            event_type: "CustomEvent".into(),
            detail: None,
        }
    }

    pub fn input(name: &str, detail: Value) -> Self {
        Self {
            name: name.into(),
            event_type: "InputEvent".into(),
            detail: Some(detail),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionOutcome {
    pub state: Value,
    pub events: Vec<DomEvent>,
}

pub fn dispatch(
    tag: &str,
    state: &mut Value,
    action: &str,
    payload: &Value,
) -> Option<ActionOutcome> {
    let events = match tag {
        "wokwi-analog-joystick" => {
            crate::components::analog_joystick::action(state, action, payload)
        }
        "wokwi-arduino-uno" | "wokwi-arduino-nano" | "wokwi-arduino-mega" | "wokwi-franzininho" => {
            crate::components::board_leds::action(state, action, payload)
        }
        "wokwi-dip-switch-8" => crate::components::dip_switch::action(state, action, payload),
        "wokwi-ky-040" => crate::components::ky040::action(state, action, payload),
        "wokwi-ir-remote" => crate::components::ir_remote::action(state, action, payload),
        "wokwi-membrane-keypad" => {
            crate::components::membrane_keypad::action(state, action, payload)
        }
        "wokwi-pushbutton" => crate::components::pushbutton::action(state, action, payload),
        "wokwi-pushbutton-6mm" => crate::components::pushbutton_6mm::action(state, action, payload),
        "wokwi-potentiometer" => crate::components::potentiometer::action(state, action, payload),
        "wokwi-rotary-dialer" => crate::components::rotary_dialer::action(state, action, payload),
        "wokwi-slide-switch" => crate::components::slide_switch::action(state, action, payload),
        "wokwi-slide-potentiometer" => {
            crate::components::slide_potentiometer::action(state, action, payload)
        }
        _ => Vec::new(),
    };

    Some(ActionOutcome {
        state: state.clone(),
        events,
    })
}

pub fn invoke(tag: &str, state: &mut Value, method: &str, args: &Value) -> bool {
    match tag {
        "wokwi-led-ring" => crate::components::led_ring::invoke(state, method, args),
        "wokwi-neopixel-matrix" => crate::components::neopixel_matrix::invoke(state, method, args),
        "wokwi-rotary-dialer" => crate::components::rotary_dialer::invoke(state, method, args),
        _ => false,
    }
}
