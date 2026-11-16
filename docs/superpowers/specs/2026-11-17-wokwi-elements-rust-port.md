# Wokwi Elements Rust Port Specification

## Goal

Reimplement `@wokwi/elements` 1.9.2 in Rust so applications can replace the
upstream JavaScript package without changing their markup, property access,
event handling, pin metadata, or visible rendering.

The compatibility oracle is the published 1.9.2 package, which is cached in the
local pnpm store. The pinned package contains 50 custom elements, their compiled
test files, TypeScript declarations, and MIT license.

## Compatibility Contract

For every upstream custom element:

- The custom-element tag name is identical.
- Public writable properties have the same names, defaults, JavaScript types,
  and attribute reflection behavior.
- `pinInfo` returns the same ordered array of `{ name, x, y, signals,
  description? }` values for each supported state.
- `pininfo-change` is emitted at the same state transitions.
- Pointer, keyboard, and touch interactions produce the same event names and
  `detail` payloads.
- Public semantic methods, canvases, and image-data APIs remain callable with the
  same shapes. Lit lifecycle/rendering helpers such as `render`, `renderSVG`, and
  helper methods returning `TemplateResult` are implementation details.
- Shadow-DOM visible markup is structurally equivalent after removing Lit
  comment markers and insignificant whitespace.
- CSS declarations and SVG geometry, colors, opacity, filters, gradients,
  classes, ARIA text, and labels are equivalent.
- Default and representative state screenshots match the upstream package.

JavaScript is allowed only as the browser bootstrap that registers custom
elements and forwards DOM callbacks to WASM. Component state, property
coercion, rendering, pin metadata, methods, and interaction semantics belong in
Rust.

## Compatibility Matrix

The 50 tag names in scope are:

`wokwi-7segment`, `wokwi-analog-joystick`, `wokwi-arduino-mega`,
`wokwi-arduino-nano`, `wokwi-arduino-uno`, `wokwi-biaxial-stepper`,
`wokwi-big-sound-sensor`, `wokwi-buzzer`, `wokwi-dht22`,
`wokwi-dip-switch-8`, `wokwi-ds1307`, `wokwi-esp32-devkit-v1`,
`wokwi-flame-sensor`, `wokwi-franzininho`, `wokwi-gas-sensor`,
`wokwi-hc-sr04`, `wokwi-heart-beat-sensor`, `wokwi-hx711`,
`wokwi-ili9341`, `wokwi-ir-receiver`, `wokwi-ir-remote`,
`wokwi-ks2e-m-dc5`, `wokwi-ky-040`, `wokwi-lcd1602`, `wokwi-lcd2004`,
`wokwi-led`, `wokwi-led-bar-graph`, `wokwi-led-ring`,
`wokwi-membrane-keypad`, `wokwi-microsd-card`, `wokwi-mpu6050`,
`wokwi-nano-rp2040-connect`, `wokwi-neopixel`,
`wokwi-neopixel-matrix`, `wokwi-ntc-temperature-sensor`,
`wokwi-photoresistor-sensor`, `wokwi-pir-motion-sensor`,
`wokwi-potentiometer`, `wokwi-pushbutton`, `wokwi-pushbutton-6mm`,
`wokwi-resistor`, `wokwi-rgb-led`, `wokwi-rotary-dialer`, `wokwi-servo`,
`wokwi-slide-potentiometer`, `wokwi-slide-switch`,
`wokwi-small-sound-sensor`, `wokwi-ssd1306`, `wokwi-stepper-motor`,
`wokwi-tilt-switch`.

## Architecture

- `wokwi-elements` is a Rust `cdylib` compiled for
  `wasm32-unknown-unknown`.
- A Rust registry maps tags to metadata and component implementations.
- A `serde_json::Value` state object mirrors the public JavaScript values.
- A thin JavaScript base class owns `HTMLElement`, shadow DOM, and event
  listeners; it delegates state transitions and rendering to exported Rust
  functions.
- SVG is emitted from Rust as strings, with strict HTML escaping and stable
  property coercion.
- Playwright and the locally cached browser render both implementations for
  differential tests.

## Non-Goals

- Hardware simulation is not part of the upstream package and is not ported.
- Lit internals, decorators, and generated DOM comment markers are
  implementation details and are not reproduced.
- Network installation is not required to run the compatibility suite because
  the pinned oracle and browser are available locally.

## Acceptance

`cargo test` passes, the WASM bundle builds, all 50 tags register, the contract
matrix reports every required property/event/API, and browser differential tests
pass for each implemented component. A component is complete only when its
default and stateful visual snapshots and its event/pin assertions pass.
