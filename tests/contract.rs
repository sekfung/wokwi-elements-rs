use serde_json::json;
use wokwi_elements_rs::pin_info;
use wokwi_elements_rs::render;
use wokwi_elements_rs::tag_names;

#[test]
fn every_component_has_a_default_render() {
    for tag in tag_names() {
        assert!(
            render(tag, &json!({})).is_some(),
            "missing default render for {tag}"
        );
    }
}

#[test]
fn exposes_every_upstream_component_tag() {
    let tags = tag_names();

    assert_eq!(tags.len(), 50);
    assert!(tags.contains(&"wokwi-7segment"));
    assert!(tags.contains(&"wokwi-arduino-uno"));
    assert!(tags.contains(&"wokwi-led"));
    assert!(tags.contains(&"wokwi-pushbutton"));
    assert!(tags.contains(&"wokwi-tilt-switch"));
}

#[test]
fn led_pin_info_follows_the_flip_property() {
    let normal = pin_info("wokwi-led", &json!({ "flip": false })).unwrap();
    assert_eq!(normal.len(), 2);
    assert_eq!(
        (normal[0].name.as_str(), normal[0].x, normal[0].y),
        ("A", 25.0, 42.0)
    );
    assert_eq!(
        (normal[1].name.as_str(), normal[1].x, normal[1].y),
        ("C", 15.0, 42.0)
    );
    assert_eq!(normal[0].description.as_deref(), Some("Anode"));
    assert_eq!(normal[1].description.as_deref(), Some("Cathode"));

    let flipped = pin_info("wokwi-led", &json!({ "flip": true })).unwrap();
    assert_eq!((flipped[0].name.as_str(), flipped[0].x), ("A", 15.0));
    assert_eq!((flipped[1].name.as_str(), flipped[1].x), ("C", 25.0));
}

#[test]
fn led_render_reflects_light_state_and_flip() {
    let lit = render(
        "wokwi-led",
        &json!({
            "value": true,
            "brightness": 0.5,
            "color": "green",
            "label": "PWR",
            "flip": true
        }),
    )
    .unwrap();

    assert!(lit.html.contains(r#"transform="scale(-1 1)""#));
    assert!(lit.html.contains(r#"class="light" style="display: ""#));
    assert!(lit.html.contains("fill=\"#80ff80\""));
    assert!(lit.html.contains(r#"style="opacity: 0.6499999999999999""#));
    assert!(lit.html.contains("PWR"));

    let dark = render("wokwi-led", &json!({ "value": false })).unwrap();
    assert!(dark.html.contains(r#"class="light" style="display: none""#));
}

#[test]
fn resistor_uses_standard_four_band_colors() {
    let resistor = render("wokwi-resistor", &json!({ "value": "220" })).unwrap();

    assert!(resistor.html.contains("fill=\"#FB0000\""));
    assert!(resistor.html.contains("fill=\"#8F4814\""));
    assert!(resistor
        .html
        .contains("<path d=\"m6 0.29411v2.4117h0.96v-2.4117z\" fill=\"#FB0000\""));
}

#[test]
fn seven_segment_lights_the_requested_segments() {
    let display = render(
        "wokwi-7segment",
        &json!({
            "digits": 1,
            "values": [1, 0, 0, 0, 0, 0, 1, 0],
            "color": "#ff0000",
            "offColor": "#333333"
        }),
    )
    .unwrap();

    assert!(display.html.contains("fill=\"#ff0000\""));
    assert!(display.html.contains("fill=\"#333333\""));
    assert_eq!(display.html.matches("<polygon").count(), 7);
}

#[test]
fn seven_segment_pin_info_changes_with_digit_count() {
    let pins = pin_info("wokwi-7segment", &json!({ "digits": 2, "pins": "top" })).unwrap();

    assert_eq!(pins.len(), 10);
    assert_eq!(pins[0].name, "DIG1");
    assert_eq!(pins[9].name, "DP");
    assert!(pins.iter().all(|pin| pin.description.is_some()));
}

#[test]
fn led_bar_graph_uses_named_palette_colors() {
    let graph = render(
        "wokwi-led-bar-graph",
        &json!({
            "color": "GYR",
            "offColor": "#111111",
            "values": [1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        }),
    )
    .unwrap();

    assert!(graph.html.contains("fill=\"#9eff3c\""));
    assert!(graph.html.contains("fill=\"#f1d73c\""));
    assert!(graph.html.contains("fill=\"#dc012d\""));
    assert_eq!(graph.html.matches("<rect x=\"2.5\"").count(), 10);
}

#[test]
fn rgb_led_maps_channel_values_to_glow_and_color() {
    let led = render(
        "wokwi-rgb-led",
        &json!({ "ledRed": 0.5, "ledGreen": 1.0, "ledBlue": 0.0 }),
    )
    .unwrap();

    assert!(led.html.contains("stdDeviation=\"1.5\""));
    assert!(led.html.contains("stdDeviation=\"3\""));
    assert!(led.html.contains("r=\"4.5\""));
    assert!(led.html.contains("r=\"7\""));
    assert!(led.html.contains("fill=\"rgb(127.5, 255, 0)\""));
    assert!(led.html.contains("opacity=\"0.8\""));
}

#[test]
fn neopixel_maps_rgb_to_background_and_glow() {
    let pixel = render("wokwi-neopixel", &json!({ "r": 1.0, "g": 0.0, "b": 0.0 })).unwrap();

    assert!(pixel.html.contains("fill=\"rgb(242, 242, 242)\""));
    assert!(pixel.html.contains("fill=\"rgb(255, 0, 0)\""));
    assert!(pixel.html.contains("opacity=\"1\""));
    assert!(pixel.html.contains("stdDeviation=\"1\""));
}

#[test]
fn simple_sensor_leds_follow_their_properties() {
    let cases = [
        (
            "wokwi-gas-sensor",
            json!({ "ledPower": true, "ledD0": true }),
            "<circle cx=\"85.5\" cy=\"8\" r=\"1.8\"",
            "<circle cx=\"85\" cy=\"50\" r=\"1.8\"",
        ),
        (
            "wokwi-flame-sensor",
            json!({ "ledPower": true, "ledSignal": true }),
            "<circle cx=\"153.2\" cy=\"7.5\" r=\"7\"",
            "<circle cx=\"153.2\" cy=\"51.3\" r=\"7\"",
        ),
        (
            "wokwi-big-sound-sensor",
            json!({ "led1": true, "led2": true }),
            "<circle cx=\"56\" cy=\"4\" r=\"5\"",
            "<circle cx=\"56\" cy=\"47\" r=\"5\"",
        ),
        (
            "wokwi-small-sound-sensor",
            json!({ "ledPower": true, "ledSignal": true }),
            "<circle cx=\"56\" cy=\"4\" r=\"7\"",
            "<circle cx=\"56\" cy=\"47\" r=\"7\"",
        ),
        (
            "wokwi-photoresistor-sensor",
            json!({ "ledPower": true, "ledDO": true }),
            "<circle cx=\"124.5\" cy=\"7\" r=\"4\"",
            "<circle cx=\"124.5\" cy=\"55\" r=\"4\"",
        ),
        (
            "wokwi-mpu6050",
            json!({ "led1": true }),
            "<circle cx=\"13.9\" cy=\"25.5\" r=\"3.5\"",
            "<circle cx=\"13.9\" cy=\"25.5\" r=\"3.5\"",
        ),
    ];

    for (tag, state, first_pin, second_pin) in cases {
        let html = render(tag, &state).unwrap().html;
        assert!(html.contains(first_pin), "{tag} missing {first_pin}");
        assert!(html.contains(second_pin), "{tag} missing {second_pin}");
    }
}

#[test]
fn dip_switch_positions_follow_values() {
    let html = render(
        "wokwi-dip-switch-8",
        &json!({ "values": [1, 0, 1, 0, 1, 0, 1, 0] }),
    )
    .unwrap()
    .html;

    assert_eq!(html.matches("y=\"-7.2\"").count(), 4);
    assert_eq!(html.matches("y=\"0\"").count(), 4);
}

#[test]
fn buzzer_shows_note_only_while_signaling() {
    let quiet = render("wokwi-buzzer", &json!({ "hasSignal": false })).unwrap();
    let ringing = render("wokwi-buzzer", &json!({ "hasSignal": true })).unwrap();

    assert!(quiet.html.contains("visibility: hidden"));
    assert!(ringing.html.contains("visibility: "));
    assert!(!ringing.html.contains("visibility: hidden"));
}

#[test]
fn servo_renders_angle_horn_shape_and_color() {
    let servo = render(
        "wokwi-servo",
        &json!({ "angle": 90, "horn": "cross", "hornColor": "#ff5500" }),
    )
    .unwrap();

    assert!(servo.html.contains("rotate(90)"));
    assert!(servo.html.contains("fill=\"#ff5500\""));
    assert!(servo.html.contains("m119.54 50.354h-18.653v-18.653"));
    assert!(!servo.html.contains("m101.6 59.589-4.3167-54.166"));
}

#[test]
fn board_leds_follow_their_properties() {
    let cases = [
        (
            "wokwi-arduino-uno",
            json!({ "ledPower": true, "led13": true, "ledTX": true, "ledRX": true }),
            "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\"",
        ),
        (
            "wokwi-arduino-nano",
            json!({ "ledPower": true, "led13": true, "ledTX": true, "ledRX": true }),
            "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\"",
        ),
        (
            "wokwi-arduino-mega",
            json!({ "ledPower": true, "led13": true, "ledTX": true, "ledRX": true }),
            "<circle cx=\"1.3\" cy=\"0.55\" r=\"1.3\"",
        ),
        (
            "wokwi-esp32-devkit-v1",
            json!({ "ledPower": true, "led1": true }),
            "<circle cx=\"37\" cy=\"112.5\" r=\"4\"",
        ),
        (
            "wokwi-franzininho",
            json!({ "ledPower": true, "led1": true }),
            "<circle cx=\"33.6\" cy=\"25.9\" r=\"1.8\"",
        ),
        (
            "wokwi-nano-rp2040-connect",
            json!({ "ledPower": true, "ledBuiltIn": true, "ledRed": 1, "ledGreen": 0, "ledBlue": 0 }),
            "<circle cx=\"14.5\" cy=\"14.5\" r=\"3\"",
        ),
    ];

    for (tag, state, marker) in cases {
        let html = render(tag, &state).unwrap().html;
        assert!(html.contains(marker), "{tag} missing {marker}");
    }
}

#[test]
fn slide_potentiometer_moves_tip_and_reports_travel_pin() {
    let low = render(
        "wokwi-slide-potentiometer",
        &json!({ "travelLength": 30, "min": 0, "max": 100, "value": 0 }),
    )
    .unwrap();
    let middle = render(
        "wokwi-slide-potentiometer",
        &json!({ "travelLength": 30, "min": 0, "max": 100, "value": 50 }),
    )
    .unwrap();
    let high = render(
        "wokwi-slide-potentiometer",
        &json!({ "travelLength": 30, "min": 0, "max": 100, "value": 100 }),
    )
    .unwrap();

    assert!(low
        .html
        .contains("id=\"tip\" transform=\"translate(-15 0)\""));
    assert!(middle
        .html
        .contains("id=\"tip\" transform=\"translate(0 0)\""));
    assert!(high
        .html
        .contains("id=\"tip\" transform=\"translate(15 0)\""));

    let pins = pin_info("wokwi-slide-potentiometer", &json!({ "travelLength": 30 })).unwrap();
    assert_eq!(pins[2].name, "GND");
    assert!((pins[2].x - (93.6 + 30.0 * 3.78)).abs() < 1e-6);
}

#[test]
fn stepper_motor_renders_nema_size_angle_and_labels() {
    let html = render(
        "wokwi-stepper-motor",
        &json!({ "size": 11, "angle": 90, "arrow": "red", "value": "42", "units": "steps" }),
    )
    .unwrap()
    .html;

    assert!(html.contains("width=\"29.2mm\""));
    assert!(html.contains("rotate(90,"));
    assert!(html.contains("fill=\"red\""));
    assert!(html.contains(">42</tspan>"));
    assert!(html.contains(">steps</tspan>"));

    let pins = pin_info("wokwi-stepper-motor", &json!({ "size": 11 })).unwrap();
    assert_eq!(pins.len(), 4);
    assert_eq!(pins[0].name, "A-");
    assert_eq!(pins[3].name, "B-");
}

#[test]
fn hx711_renders_all_three_load_sensor_variants() {
    let fifty = render("wokwi-hx711", &json!({ "type": "50kg" })).unwrap();
    let five = render("wokwi-hx711", &json!({ "type": "5kg" })).unwrap();
    let gauge = render("wokwi-hx711", &json!({ "type": "gauge" })).unwrap();

    assert!(fifty.html.contains("50kg Load Sensor"));
    assert!(five.html.contains("5kg Load Sensor"));
    assert!(gauge.html.contains("Gauge Pressure Sensor"));
    assert_ne!(fifty.html, five.html);
    assert_ne!(five.html, gauge.html);
}

#[test]
fn biaxial_stepper_renders_hand_shapes_angles_colors_and_lengths() {
    let html = render(
        "wokwi-biaxial-stepper",
        &json!({
            "outerHandShape": "arrow",
            "outerHandLength": 42,
            "outerHandAngle": 90,
            "outerHandColor": "red",
            "innerHandShape": "ornate",
            "innerHandLength": 28,
            "innerHandAngle": -30,
            "innerHandColor": "blue"
        }),
    )
    .unwrap()
    .html;

    assert!(html.contains("rotate(360)"));
    assert!(html.contains("rotate(240)"));
    assert!(html.contains("fill=\"red\""));
    assert!(html.contains("fill=\"blue\""));
    assert!(html.contains("c 3.89 0 7.16 -2.42 8.43 -5.85"));
    assert!(html.contains("c 0 5 5 5 5 5"));
    assert!(html.contains("l 42 -4.7"));
}

#[test]
fn lcd1602_renders_text_pins_cursor_and_screen_only_mode() {
    let html = render(
        "wokwi-lcd1602",
        &json!({
            "text": "Hi",
            "background": "blue",
            "pins": "i2c",
            "screenOnly": true,
            "cursor": true,
            "cursorX": 2,
            "cursorY": 1,
            "backlight": false
        }),
    )
    .unwrap()
    .html;

    assert!(html.contains("fill=\"#000eff\""));
    assert!(html.contains("GND</tspan>"));
    assert!(html.contains("SDA</tspan>"));
    assert!(html.contains("fill=\"black\""));
    assert!(html.contains("viewBox=\"12.45 12.55"));
    assert!(html.contains("<path d=\"M 0.00 0.00h0.55v0.65h-0.55Z"));

    let pins = pin_info("wokwi-lcd1602", &json!({ "pins": "i2c" })).unwrap();
    assert_eq!(pins.len(), 4);
    assert_eq!(pins[2].name, "SDA");
}

#[test]
fn lcd2004_uses_twenty_by_four_geometry() {
    let display = render(
        "wokwi-lcd2004",
        &json!({ "text": "Boardless", "pins": "full" }),
    )
    .unwrap();
    assert!(display.html.contains("viewBox=\"0 0 94.05 47.5\""));
    assert!(display.html.contains("width=\"70.25\""));
    assert!(display.html.contains("height=\"23\""));

    let pins = pin_info("wokwi-lcd2004", &json!({ "pins": "full" })).unwrap();
    assert_eq!(pins.len(), 16);
    assert_eq!(pins[0].name, "VSS");
    assert_eq!(pins[15].name, "K");
}

#[test]
fn membrane_keypad_renders_three_columns_connector_and_custom_keys() {
    let keys = [
        "A", "b", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P",
    ];
    let html = render(
        "wokwi-membrane-keypad",
        &json!({ "columns": "3", "connector": true, "keys": keys }),
    )
    .unwrap()
    .html;

    assert!(html.contains("width=\"55.336mm\""));
    assert!(html.contains("height=\"91mm\""));
    assert!(html.contains("url(#pins-female)"));
    assert_eq!(html.matches("data-key-name=").count(), 12);
    assert!(html.contains(">A</text>"));
    assert!(html.contains(">b</text>"));

    let pins = pin_info("wokwi-membrane-keypad", &json!({ "columns": "3" })).unwrap();
    assert_eq!(pins.len(), 7);
    assert_eq!(pins.last().unwrap().name, "C3");
}

#[test]
fn potentiometer_maps_value_to_knob_angle() {
    let low = render(
        "wokwi-potentiometer",
        &json!({ "min": 0, "max": 1023, "value": 0, "startDegree": -135, "endDegree": 135 }),
    )
    .unwrap();
    let middle = render(
        "wokwi-potentiometer",
        &json!({ "min": 0, "max": 1023, "value": 511.5, "startDegree": -135, "endDegree": 135 }),
    )
    .unwrap();
    let high = render(
        "wokwi-potentiometer",
        &json!({ "min": 0, "max": 1023, "value": 1023, "startDegree": -135, "endDegree": 135 }),
    )
    .unwrap();

    assert!(low.html.contains("--knob-angle: -135deg"));
    assert!(middle.html.contains("--knob-angle: 0deg"));
    assert!(high.html.contains("--knob-angle: 135deg"));
    assert!(high.html.contains("aria-valuenow=\"1023\""));
}
