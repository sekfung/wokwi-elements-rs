# Wokwi Elements — Rust/WASM port

This repository is a Rust reimplementation of
[`@wokwi/elements`](https://github.com/wokwi/wokwi-elements), based on the published
`1.9.2` package. Component state, rendering, events, and public methods live in Rust;
the browser file is a thin custom-element and DOM bridge around the generated WASM.

## Compatibility baseline

- Reference package: `@wokwi/elements@1.9.2`
- Component tags registered: **50 / 50**
- Default-state pixel parity: **50 / 50**
- API/default/pin metadata parity: **50 / 50**
- Property-state pixel parity: **44 sampled states across 37 components**
- Dedicated Rust renderers: **40**, plus 10 no-reactive-property components using pinned static assets
- Semantic method API coverage: LED ring, NeoPixel matrix, LCD text/font, rotary dial, SSD1306, ILI9341

The verified compatibility matrix and test coverage are listed in
[`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md).

## Build

Requirements: Rust with the `wasm32-unknown-unknown` target, `wasm-bindgen` `0.2.130`,
and Node.js 20 or newer for browser tests.

```bash
bash scripts/build.sh
```

The browser entry point is `web/index.js`; generated WASM files are written to
`web/pkg/` and are intentionally excluded from source control.

### Native Boardless artwork pack

Build the import-free native guest with the pinned Rust toolchain and lockfile.

```bash
rustup target add --toolchain 1.95.0 wasm32-unknown-unknown
cargo +1.95.0 build --locked --release --target wasm32-unknown-unknown --features native-pack
cargo +1.95.0 test --locked --features native-pack
```

The artifact is `target/wasm32-unknown-unknown/release/wokwi_elements_rs.wasm`.
It exports memory and Artwork ABI 1. The native host writes UTF-8 JSON into
the buffer from `element_input(len)`, then reads the result descriptor from
`element_call()`. The descriptor encodes the pointer in the high 32 bits and
the byte length in the low 32 bits. Zero reports an invalid request. Copy the
bytes before calling another export. Requests are limited to 64 KiB and
responses to 4 MiB.

- `{"op":"catalog"}` returns entries with `tag` and `default_state`.
- `{"op":"render","tag":"wokwi-led","state":{}}` returns `html`, `styles`, and
  `pins`.

Build without `native-pack` for the browser bindings.

### Interactive component gallery

```bash
npm run dev
```

Open `http://127.0.0.1:4173/web/dev.html`. The gallery provides a searchable
component catalog, automatic property controls, event log, `pinInfo` inspector,
zoom/grid controls, and a generated API snippet.

`npm run dev` uses a debug WASM build with Rust symbols and DWARF debug sections.
Open Chrome DevTools → Sources, choose the WebAssembly target, and set breakpoints
in Rust functions such as `render_component`, `dispatch_action`, or a component
renderer. Panics are forwarded to the browser console through
`console_error_panic_hook`.

For native Rust breakpoints or `dbg!`, inspect a component state without a browser:

```bash
cargo run --offline --example inspect -- wokwi-led '{"value":true,"color":"green"}'
```

This prints the Rust-rendered HTML, CSS, and `pinInfo`, making it straightforward
to debug with `lldb`, `rust-gdb`, or an IDE Rust debugger.

```html
<script type="module">
  import '@wokwi/elements-rs';

  const led = document.createElement('wokwi-led');
  led.value = true;
  document.body.append(led);
</script>
```

## Test

Rust contract tests:

```bash
cargo test --offline
```

Browser contract tests:

```bash
NODE_PATH=/path/to/playwright/node_modules node tests/browser-smoke.cjs
NODE_PATH=/path/to/playwright/node_modules node tests/browser-visual-parity.cjs
NODE_PATH=/path/to/playwright/node_modules node tests/browser-property-parity.cjs
NODE_PATH=/path/to/playwright/node_modules node tests/browser-api-parity.cjs
```

The visual parity test renders every component with the upstream package and the
Rust/WASM build in Chromium, then compares screenshots pixel by pixel.
The API and property parity tests additionally compare observed attributes,
property defaults, `pinInfo`, representative state screenshots, and public
interaction/event behavior.

Lit lifecycle methods and rendering helper methods returning `TemplateResult`
are treated as internal implementation details rather than consumer API.

## Architecture

- `src/contract.rs` loads the captured public property contract.
- `src/generated/` stores default render and pin fixtures used for visual parity.
- `src/components/` contains component state machines and renderers.
- `src/events.rs` converts DOM actions into state changes and emitted events.
- `web/index.js` defines the custom elements, property accessors, DOM morphing,
  event delegation, and public method shims.
- `reference/` contains the MIT-licensed upstream `1.9.2` package used only as a
  test oracle.

## License

MIT. The upstream Wokwi Elements license and source attribution are preserved in
`reference/wokwi-elements-1.9.2/`.
