# Wokwi Elements Rust Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reimplement all 50 `@wokwi/elements` 1.9.2 custom elements in Rust/WASM with one-to-one API, metadata, interaction, and visual compatibility.

**Architecture:** Rust owns the component registry, typed values, state transitions, pin metadata, event effects, and SVG rendering. A small JavaScript class extends `HTMLElement`, owns shadow DOM, and forwards callbacks into WASM. The published 1.9.2 package in the local pnpm cache is copied into `reference/` as the immutable compatibility oracle.

**Tech Stack:** Rust 1.99, `wasm32-unknown-unknown`, `wasm-bindgen` 0.2.130, `serde_json` 1.0.154, Node 24, Playwright 1.64.1, Chromium 1248.

**Spec:** `docs/superpowers/specs/2026-11-17-wokwi-elements-rust-port.md`

## Global Constraints

- Every custom-element tag in the spec remains unchanged.
- The compatibility oracle is `@wokwi/elements@1.9.2`.
- Component logic belongs in Rust; JavaScript is limited to a browser bridge.
- Every production behavior starts with a failing test.
- Tests use the real Rust crate and real browser DOM, not mocks.
- Reference files are test fixtures only and must never be imported by the Rust runtime.
- No hardware simulation code is added.

---

### Task 1: Pin The Oracle And Establish The Red Test

**Files:**
- Create: `reference/README.md`
- Create: `reference/wokwi-elements/` by copying the cached package
- Create: `tests/registry.rs`
- Create: `src/lib.rs`
- Create: `Cargo.toml`

**Interfaces:**
- Consumes: none.
- Produces: `wokwi_elements::registry::tags() -> &'static [&'static str]` and `wokwi_elements::registry::lookup(tag: &str) -> Option<&'static ComponentSpec>`.

- [ ] **Step 1: Copy the immutable oracle**

Run:

```bash
cp -a /tmp/boardless-electron-desktop/node_modules/.pnpm/@wokwi+elements@1.9.2/node_modules/@wokwi/elements reference/wokwi-elements
```

Write `reference/README.md` stating the version, source path, MIT license, and that this directory is test-only.

- [ ] **Step 2: Write the failing registry test**

`tests/registry.rs`:

```rust
use wokwi_elements::registry;

#[test]
fn registers_every_upstream_tag_once() {
    let expected = [
        "wokwi-7segment", "wokwi-analog-joystick", "wokwi-arduino-mega",
        "wokwi-arduino-nano", "wokwi-arduino-uno", "wokwi-biaxial-stepper",
        "wokwi-big-sound-sensor", "wokwi-buzzer", "wokwi-dht22",
        "wokwi-dip-switch-8", "wokwi-ds1307", "wokwi-esp32-devkit-v1",
        "wokwi-flame-sensor", "wokwi-franzininho", "wokwi-gas-sensor",
        "wokwi-hc-sr04", "wokwi-heart-beat-sensor", "wokwi-hx711",
        "wokwi-ili9341", "wokwi-ir-receiver", "wokwi-ir-remote",
        "wokwi-ks2e-m-dc5", "wokwi-ky-040", "wokwi-lcd1602",
        "wokwi-lcd2004", "wokwi-led", "wokwi-led-bar-graph",
        "wokwi-led-ring", "wokwi-membrane-keypad", "wokwi-microsd-card",
        "wokwi-mpu6050", "wokwi-nano-rp2040-connect", "wokwi-neopixel",
        "wokwi-neopixel-matrix", "wokwi-ntc-temperature-sensor",
        "wokwi-photoresistor-sensor", "wokwi-pir-motion-sensor",
        "wokwi-potentiometer", "wokwi-pushbutton", "wokwi-pushbutton-6mm",
        "wokwi-resistor", "wokwi-rgb-led", "wokwi-rotary-dialer",
        "wokwi-servo", "wokwi-slide-potentiometer", "wokwi-slide-switch",
        "wokwi-small-sound-sensor", "wokwi-ssd1306", "wokwi-stepper-motor",
        "wokwi-tilt-switch",
    ];

    let mut actual = registry::tags().to_vec();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}
```

- [ ] **Step 3: Verify RED**

Run: `cargo test --test registry`

Expected: compilation fails because the crate and registry module do not exist.

- [ ] **Step 4: Add the minimal crate and registry**

Create a Rust library crate and the 51 sorted tag entries. Each entry initially
uses a private placeholder renderer that returns an empty string.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --test registry`

Expected: one passing test with no warnings.

---

### Task 2: Rust WASM Bridge And Property Coercion

**Files:**
- Create: `src/runtime.rs`
- Create: `src/render.rs`
- Create: `web/wokwi-elements.js`
- Create: `web/index.html`
- Create: `tests/runtime.rs`

**Interfaces:**
- Consumes: `registry::lookup`.
- Produces: `runtime::State`, `runtime::set_property`, `runtime::set_attribute`, `runtime::render`, and WASM exports `component_metadata`, `create_state`, `set_value`, `render_component`.

- [ ] **Step 1: Write failing coercion and escaping tests**

Tests assert that `"false"` becomes boolean `false` for Boolean properties,
numeric attributes become JSON numbers, array properties remain arrays, labels
escape `<`, `&`, and `"`, and unknown attributes do not appear in state.

- [ ] **Step 2: Verify RED**

Run: `cargo test --test runtime`

Expected: failures report missing runtime functions.

- [ ] **Step 3: Implement minimal coercion and rendering**

Use parent elements from the spec. Implement one `wokwi-led` renderer sufficient
to exercise string, number, boolean, and null values. Add the `HTMLElement`
bridge and register only `wokwi-led`.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test --test runtime`

Expected: all coercion and escaping tests pass.

---

### Task 3: LED Family Differential Slice

**Files:**
- Create: `src/components/led.rs`
- Create: `tests/visual-led.spec.mjs`
- Modify: `src/components/mod.rs`

**Interfaces:**
- Consumes: `runtime::State` and `render::attrs`.
- Produces: implementations for `wokwi-led`, `wokwi-rgb-led`, `wokwi-led-bar-graph`, and `wokwi-led-ring`.

- [ ] **Step 1: Write the failing browser test**

For each LED tag, render default, lit, dim, custom-color, and flipped states in
the upstream reference page and the Rust page. Compare normalized shadow markup
and screenshots with a 0.1% pixel threshold.

- [ ] **Step 2: Verify RED**

Run: `node tests/visual-led.spec.mjs`

Expected: Rust tags are missing or render empty shadow roots.

- [ ] **Step 3: Port the four renderers and pin metadata**

Reproduce SVG dimensions, gradients, filters, labels, opacities, and property
defaults from the pinned modules.

- [ ] **Step 4: Verify GREEN**

Run: `node tests/visual-led.spec.mjs`

Expected: all state comparisons pass.

---

### Task 4: Button And Switch Differential Slice

**Files:**
- Create: `src/components/buttons.rs`
- Create: `tests/visual-buttons.spec.mjs`
- Modify: `src/components/mod.rs`

**Interfaces:**
- Consumes: `runtime::dispatch`.
- Produces: `wokwi-pushbutton`, `wokwi-pushbutton-6mm`, `wokwi-slide-switch`, `wokwi-dip-switch-8`, and `wokwi-tilt-switch`.

- [ ] **Step 1: Write failing interaction tests**

Tests dispatch pointer, touch, and keyboard events and assert upstream-compatible
`press`, `release`, `input`, and `switch-change` events, including `detail`
payloads and sticky behavior.

- [ ] **Step 2: Verify RED**

Run: `node tests/visual-buttons.spec.mjs`

Expected: no Rust event reaches the host element.

- [ ] **Step 3: Implement event effects and visuals**

Encode event actions in data attributes during Rust rendering and convert them
to host events in the JavaScript bridge exactly once.

- [ ] **Step 4: Verify GREEN**

Run: `node tests/visual-buttons.spec.mjs`

Expected: event sequences and screenshots match.

---

### Task 5: Analog Controls And Sensors

**Files:**
- Create: `src/components/analog.rs`
- Create: `src/components/sensors.rs`
- Create: `tests/visual-analog-sensors.spec.mjs`
- Modify: `src/components/mod.rs`

**Interfaces:**
- Consumes: runtime rendering and event effects.
- Produces: potentiometers, rotary dialer, joystick, light/sound/flame/gas/temperature/motion sensors, DHT22, DS1307, MPU6050, HX711, HC-SR04, IR receiver/remote, and servo.

- [ ] **Step 1: Write failing state and pin tests**

Cover angle extremes, joystick axes, sensor enable states, `pinInfo` snapshots,
IR button details, and input event details.

- [ ] **Step 2: Verify RED**

Run: `node tests/visual-analog-sensors.spec.mjs`

Expected: missing tags and mismatched pin arrays.

- [ ] **Step 3: Implement the family**

Port all render bodies, defaults, pin arrays, and event effects from the pinned
modules.

- [ ] **Step 4: Verify GREEN**

Run: `node tests/visual-analog-sensors.spec.mjs`

Expected: every state and pin assertion passes.

---

### Task 6: Displays, Matrices, And Canvases

**Files:**
- Create: `src/components/displays.rs`
- Create: `src/components/matrix.rs`
- Create: `tests/visual-displays.spec.mjs`
- Modify: `web/wokwi-elements.js`
- Modify: `src/components/mod.rs`

**Interfaces:**
- Consumes: runtime rendering, methods, and canvas access.
- Produces: `wokwi-7segment`, `wokwi-lcd1602`, `wokwi-lcd2004`, `wokwi-ssd1306`, `wokwi-ili9341`, `wokwi-neopixel`, `wokwi-neopixel-matrix`, `wokwi-membrane-keypad`, and `wokwi-microsd-card`.

- [ ] **Step 1: Write failing API and visual tests**

Cover digit strings, cursor/blink states, pixel drawing, image data, keypad key
details, display canvases, `canvas-ready`, and `pininfo-change`.

- [ ] **Step 2: Verify RED**

Run: `node tests/visual-displays.spec.mjs`

Expected: components and methods are missing.

- [ ] **Step 3: Implement renderers and methods**

Port drawing algorithms and expose owned canvases through the bridge without
moving rendering logic into JavaScript.

- [ ] **Step 4: Verify GREEN**

Run: `node tests/visual-displays.spec.mjs`

Expected: methods, events, and canvas pixels match.

---

### Task 7: Development Boards

**Files:**
- Create: `src/components/boards.rs`
- Create: `tests/visual-boards.spec.mjs`
- Modify: `src/components/mod.rs`

**Interfaces:**
- Consumes: runtime rendering, events, and pin metadata.
- Produces: Arduino Uno/Nano/Mega, ESP32 DevKit V1, Franzininho, and Nano RP2040 Connect.

- [ ] **Step 1: Write failing board tests**

Assert full pin arrays, reset button event details, LED states, flip/xray
rendering, and default screenshots.

- [ ] **Step 2: Verify RED**

Run: `node tests/visual-boards.spec.mjs`

Expected: board tags and pin arrays are missing.

- [ ] **Step 3: Port board SVGs and interactions**

Move the large static SVG bodies into Rust string templates, preserving every
geometry attribute and state interpolation.

- [ ] **Step 4: Verify GREEN**

Run: `node tests/visual-boards.spec.mjs`

Expected: all board comparisons pass.

---

### Task 8: Contract Matrix, Packaging, And Release Verification

**Files:**
- Create: `tests/contract.spec.mjs`
- Create: `scripts/build.sh`
- Create: `README.md`
- Create: `THIRD_PARTY.md`
- Modify: `package.json`

**Interfaces:**
- Consumes: all components.
- Produces: a loadable ESM package and documented compatibility matrix.

- [ ] **Step 1: Write the failing contract test**

Load every tag from the oracle, enumerate tag names, defaults, observed
properties, pin arrays, methods, and event names, then assert equality against
the Rust package.

- [ ] **Step 2: Verify RED**

Run: `node tests/contract.spec.mjs`

Expected: report at least one API mismatch.

- [ ] **Step 3: Close mismatches and add build/package files**

Add deterministic WASM build output, ESM bootstrap, MIT notices, usage
instructions, and the generated compatibility matrix.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
cargo test
./scripts/build.sh
node tests/contract.spec.mjs
node tests/visual-led.spec.mjs
node tests/visual-buttons.spec.mjs
node tests/visual-analog-sensors.spec.mjs
node tests/visual-displays.spec.mjs
node tests/visual-boards.spec.mjs
```

Expected: all commands exit zero and the generated report lists 51/51 complete.

---

## Self-Review

- Spec coverage: tags, properties, attributes, pin arrays, events, methods,
  canvases, markup, CSS, screenshots, packaging, and license are assigned.
- Placeholder scan: no deferred behavior or unnamed files remain.
- Type consistency: registry lookup, JSON state, event effects, and render
  signatures are shared across all tasks.
