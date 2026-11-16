# Compatibility Matrix

Baseline: `@wokwi/elements@1.9.2`.

## Verified now

| Area | Status |
| --- | --- |
| Component tag registration | 50 / 50 |
| Default-state screenshot parity | 50 / 50 |
| Observed attributes and property defaults | 50 / 50 |
| Default `pinInfo` and sampled stateful `pinInfo` | Passing |
| Property-state screenshot parity | 44 states across 37 components |
| Rust/WASM browser bootstrap | Passing |
| DOM-preserving property updates | Passing |
| Event delegation, custom events, `pininfo-change` | Passing |
| Canvas/image-data and method APIs | Passing |

## Stateful Rust implementations

The following tags no longer depend only on the captured default renderer:

- `wokwi-7segment`
- `wokwi-analog-joystick`
- `wokwi-arduino-mega`
- `wokwi-arduino-nano`
- `wokwi-arduino-uno`
- `wokwi-big-sound-sensor`
- `wokwi-biaxial-stepper`
- `wokwi-buzzer`
- `wokwi-dip-switch-8`
- `wokwi-esp32-devkit-v1`
- `wokwi-flame-sensor`
- `wokwi-franzininho`
- `wokwi-gas-sensor`
- `wokwi-hx711`
- `wokwi-ili9341`
- `wokwi-ky-040`
- `wokwi-ir-remote`
- `wokwi-lcd1602`
- `wokwi-lcd2004`
- `wokwi-led`
- `wokwi-led-bar-graph`
- `wokwi-led-ring`
- `wokwi-membrane-keypad`
- `wokwi-mpu6050`
- `wokwi-neopixel`
- `wokwi-neopixel-matrix`
- `wokwi-nano-rp2040-connect`
- `wokwi-photoresistor-sensor`
- `wokwi-potentiometer`
- `wokwi-pushbutton`
- `wokwi-pushbutton-6mm`
- `wokwi-resistor`
- `wokwi-rgb-led`
- `wokwi-rotary-dialer`
- `wokwi-servo`
- `wokwi-slide-potentiometer`
- `wokwi-slide-switch`
- `wokwi-small-sound-sensor`
- `wokwi-ssd1306`
- `wokwi-stepper-motor`

The ten no-reactive-property components use pinned default render and pin
fixtures. Lit lifecycle and rendering helper methods are excluded because they
return Lit `TemplateResult` implementation types and are not consumer APIs.

Static-asset components are `wokwi-dht22`, `wokwi-ds1307`, `wokwi-hc-sr04`,
`wokwi-heart-beat-sensor`, `wokwi-ir-receiver`, `wokwi-ks2e-m-dc5`,
`wokwi-microsd-card`, `wokwi-ntc-temperature-sensor`,
`wokwi-pir-motion-sensor`, and `wokwi-tilt-switch`.

## Interaction coverage

Browser tests exercise pushbuttons, DIP and slide switches,
joystick axes/press, potentiometer input, slide-potentiometer dragging, KY-040
long press, membrane-keypad press/release and Shift release, IR remote codes,
rotary dial transitions, board reset buttons, LED/NeoPixel methods and animation,
LCD text/character synchronization, SSD1306 image data/redraw, and ILI9341 canvas
readiness and flipping.

## Hardening beyond the current matrix

Exhaustive fuzzing of every possible string/array pixel combination is outside
the fixed compatibility matrix, but all public API fields, defaults, pin arrays,
representative state transitions, methods, events, and default rendering are
verified against the pinned upstream package.
