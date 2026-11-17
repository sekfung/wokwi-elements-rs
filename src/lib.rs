//! Rust implementation of the Wokwi Elements public contract.
mod components;
mod contract;
mod events;
mod generated;
mod metadata;
#[cfg(feature = "native-pack")]
mod native;
mod pins;
mod render;
mod util;
mod web;

pub use events::{ActionOutcome, DomEvent};
pub use metadata::tag_names;
pub use pins::PinInfo;
pub use render::Rendered;
pub use web::{
    component_contracts, default_state_json, install_debug_hook, invoke_method, pin_info_json,
    render_component,
};

pub fn pin_info(tag: &str, state: &serde_json::Value) -> Option<Vec<PinInfo>> {
    match tag {
        "wokwi-led" => Some(components::led::pin_info(state)),
        "wokwi-led-ring" => Some(components::led_ring::pin_info(state)),
        "wokwi-lcd1602" | "wokwi-lcd2004" => Some(components::lcd::pin_info(tag, state)),
        "wokwi-membrane-keypad" => Some(components::membrane_keypad::pin_info(state)),
        "wokwi-neopixel-matrix" => Some(components::neopixel_matrix::pin_info(state)),
        "wokwi-slide-potentiometer" => Some(components::slide_potentiometer::pin_info(state)),
        "wokwi-stepper-motor" => Some(components::stepper_motor::pin_info(state)),
        "wokwi-7segment" => Some(components::seven_segment::pin_info(state)),
        _ => generated::pin_info(tag),
    }
}

pub fn render(tag: &str, state: &serde_json::Value) -> Option<Rendered> {
    match tag {
        "wokwi-led" => Some(components::led::render(state)),
        "wokwi-analog-joystick" => Some(components::analog_joystick::render(state)),
        "wokwi-arduino-uno"
        | "wokwi-arduino-nano"
        | "wokwi-arduino-mega"
        | "wokwi-esp32-devkit-v1"
        | "wokwi-franzininho" => Some(components::board_leds::render(tag, state)),
        "wokwi-biaxial-stepper" => Some(components::biaxial_stepper::render(state)),
        "wokwi-ky-040" => Some(components::ky040::render(state)),
        "wokwi-ir-remote" => Some(components::ir_remote::render(state)),
        "wokwi-hx711" => Some(components::hx711::render(state)),
        "wokwi-ili9341" => Some(components::ili9341::render(state)),
        "wokwi-ssd1306" => Some(components::ssd1306::render(state)),
        "wokwi-nano-rp2040-connect" => Some(components::nano_rp2040::render(state)),
        "wokwi-buzzer" => Some(components::buzzer::render(state)),
        "wokwi-dip-switch-8" => Some(components::dip_switch::render(state)),
        "wokwi-big-sound-sensor"
        | "wokwi-flame-sensor"
        | "wokwi-gas-sensor"
        | "wokwi-mpu6050"
        | "wokwi-photoresistor-sensor"
        | "wokwi-small-sound-sensor" => Some(components::sensor_leds::render(tag, state)),
        "wokwi-led-bar-graph" => Some(components::led_bar_graph::render(state)),
        "wokwi-led-ring" => Some(components::led_ring::render(state)),
        "wokwi-lcd1602" | "wokwi-lcd2004" => Some(components::lcd::render(tag, state)),
        "wokwi-membrane-keypad" => Some(components::membrane_keypad::render(state)),
        "wokwi-neopixel" => Some(components::neopixel::render(state)),
        "wokwi-neopixel-matrix" => Some(components::neopixel_matrix::render(state)),
        "wokwi-pushbutton" => Some(components::pushbutton::render(state)),
        "wokwi-pushbutton-6mm" => Some(components::pushbutton_6mm::render(state)),
        "wokwi-potentiometer" => Some(components::potentiometer::render(state)),
        "wokwi-resistor" => Some(components::resistor::render(state)),
        "wokwi-rotary-dialer" => Some(components::rotary_dialer::render(state)),
        "wokwi-servo" => Some(components::servo::render(state)),
        "wokwi-rgb-led" => Some(components::rgb_led::render(state)),
        "wokwi-slide-switch" => Some(components::slide_switch::render(state)),
        "wokwi-slide-potentiometer" => Some(components::slide_potentiometer::render(state)),
        "wokwi-stepper-motor" => Some(components::stepper_motor::render(state)),
        "wokwi-7segment" => Some(components::seven_segment::render(state)),
        _ => generated::render(tag),
    }
}
