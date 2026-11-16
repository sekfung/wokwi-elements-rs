#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const samples = {
  'wokwi-7segment': [
    { digits: 2, values: [1,1,1,1,1,1,1,1,0,1,0,1,0,1,0,1], colon: true, colonValue: true, color: '#00ff00', offColor: '#222222' },
    { digits: 4, pins: 'extend', values: Array(32).fill(1) },
  ],
  'wokwi-analog-joystick': [{ xValue: 1, yValue: -1, pressed: true }],
  'wokwi-arduino-uno': [{ led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true }],
  'wokwi-arduino-nano': [{ led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true }],
  'wokwi-arduino-mega': [{ led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true }],
  'wokwi-biaxial-stepper': [
    { outerHandShape: 'arrow', outerHandAngle: 90, outerHandColor: 'red', outerHandLength: 42, innerHandShape: 'ornate', innerHandAngle: -30, innerHandColor: 'blue', innerHandLength: 28 },
    { outerHandShape: 'ornate', outerHandLength: 70, innerHandShape: 'plain', innerHandLength: 20 },
  ],
  'wokwi-big-sound-sensor': [{ led1: true, led2: true }],
  'wokwi-buzzer': [{ hasSignal: true }],
  'wokwi-dip-switch-8': [{ values: [1,0,1,0,1,0,1,0] }],
  'wokwi-esp32-devkit-v1': [{ led1: true, ledPower: true }],
  'wokwi-flame-sensor': [{ ledPower: true, ledSignal: true }],
  'wokwi-franzininho': [{ led1: true, ledPower: true, resetPressed: true }],
  'wokwi-gas-sensor': [{ ledPower: true, ledD0: true }],
  'wokwi-hx711': [{ type: '5kg' }, { type: 'gauge' }],
  'wokwi-ky-040': [{ angle: 90, pressed: true }],
  'wokwi-lcd1602': [
    { text: 'Hello, Wokwi!', background: 'blue', pins: 'i2c', backlight: false },
    { text: 'Cursor', cursor: true, cursorX: 3, cursorY: 1, screenOnly: true },
  ],
  'wokwi-lcd2004': [{ text: 'Boardless 20x4 display', pins: 'i2c', background: '#123456' }],
  'wokwi-led': [{ value: true, brightness: 0.5, color: 'green', label: 'PWR', flip: true }],
  'wokwi-led-bar-graph': [{ color: 'GYR', offColor: '#111111', values: [1,1,1,1,1,1,1,1,1,1] }],
  'wokwi-led-ring': [{ pixels: 8, pixelSpacing: 1, background: '#123456' }],
  'wokwi-membrane-keypad': [
    { columns: '3', connector: true, keys: ['A','b','C','D','E','F','G','H','I','J','K','L','M','N','O','P'] },
    { keys: ['1','2','3','+','4','5','6','-','7','8','9','*','C','0','=','/'] },
  ],
  'wokwi-mpu6050': [{ led1: true }],
  'wokwi-nano-rp2040-connect': [{ ledRed: 1, ledGreen: 0.5, ledBlue: 0, ledPower: true, ledBuiltIn: true }],
  'wokwi-neopixel': [{ r: 1, g: 0.5, b: 0 }],
  'wokwi-neopixel-matrix': [{ rows: 2, cols: 3, rowSpacing: 1.5, colSpacing: 0.5, blurLight: true }],
  'wokwi-photoresistor-sensor': [{ ledPower: true, ledDO: true }],
  'wokwi-potentiometer': [{ value: 1023, min: 0, max: 1023, startDegree: -90, endDegree: 90 }],
  'wokwi-pushbutton': [{ color: 'blue', label: 'GO', xray: true, pressed: true }],
  'wokwi-pushbutton-6mm': [{ color: 'green', label: '6mm', xray: true, pressed: true }],
  'wokwi-resistor': [{ value: '220' }, { value: '1000000' }],
  'wokwi-rgb-led': [{ ledRed: 0.5, ledGreen: 1, ledBlue: 0.25 }],
  'wokwi-servo': [{ angle: 90, horn: 'cross', hornColor: '#ff5500' }, { angle: -45, horn: 'double' }],
  'wokwi-slide-potentiometer': [{ travelLength: 45, value: 50, min: 0, max: 100 }],
  'wokwi-slide-switch': [{ value: 1 }],
  'wokwi-small-sound-sensor': [{ ledPower: true, ledSignal: true }],
  'wokwi-ssd1306': [],
  'wokwi-stepper-motor': [{ size: 11, angle: 90, arrow: 'red', value: '42', units: 'steps' }, { size: 34, angle: -90 }],
};

function contentType(filePath) {
  if (filePath.endsWith('.html')) return 'text/html; charset=utf-8';
  if (filePath.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.wasm')) return 'application/wasm';
  if (filePath.endsWith('.json')) return 'application/json; charset=utf-8';
  return 'application/octet-stream';
}

function createServer() {
  return http.createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const filePath = path.resolve(root, `.${pathname}`);
    if (!filePath.startsWith(root)) { response.writeHead(403).end(); return; }
    fs.readFile(filePath, (error, data) => {
      if (error) { response.writeHead(404).end(); return; }
      response.writeHead(200, { 'content-type': contentType(filePath) });
      response.end(data);
    });
  });
}

async function renderState(page, tag, state) {
  await page.evaluate(async ({ componentTag, properties }) => {
    document.body.replaceChildren();
    const element = document.createElement(componentTag);
    for (const [name, value] of Object.entries(properties)) element[name] = value;
    document.body.append(element);
    await element.updateComplete;
  }, { componentTag: tag, properties: state });
}

async function comparePngs(page, leftBase64, rightBase64) {
  return page.evaluate(async ({ left, right }) => {
    const decode = async (base64) => {
      const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
      const canvas = document.createElement('canvas');
      canvas.width = bitmap.width;
      canvas.height = bitmap.height;
      const context = canvas.getContext('2d', { willReadFrequently: true });
      context.drawImage(bitmap, 0, 0);
      return { width: bitmap.width, height: bitmap.height, data: context.getImageData(0, 0, bitmap.width, bitmap.height).data };
    };
    const a = await decode(left);
    const b = await decode(right);
    if (a.width !== b.width || a.height !== b.height) return { differentPixels: Infinity, maxChannelDelta: 255, dimensions: [a.width, a.height, b.width, b.height] };
    let differentPixels = 0;
    let maxChannelDelta = 0;
    for (let i = 0; i < a.data.length; i += 4) {
      let different = false;
      for (let c = 0; c < 4; c += 1) {
        const delta = Math.abs(a.data[i + c] - b.data[i + c]);
        maxChannelDelta = Math.max(maxChannelDelta, delta);
        if (delta) different = true;
      }
      if (different) differentPixels += 1;
    }
    return { differentPixels, maxChannelDelta, dimensions: [a.width, a.height] };
  }, { left: leftBase64, right: rightBase64 });
}

async function main() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  const browser = await chromium.launch({ headless: true });
  try {
    const upstream = await browser.newPage({ viewport: { width: 1200, height: 1200 } });
    const rust = await browser.newPage({ viewport: { width: 1200, height: 1200 } });
    const errors = [];
    rust.on('pageerror', (error) => errors.push(error.message));
    await upstream.goto(`http://127.0.0.1:${port}/reference/upstream-render.html`);
    await upstream.waitForFunction(() => window.elements);
    await rust.goto(`http://127.0.0.1:${port}/tests/browser-fixture.html`);
    await rust.waitForFunction(() => window.wokwiReady === true);
    const freezeAnimations = '* { animation: none !important; transition: none !important; }';
    await upstream.addStyleTag({ content: freezeAnimations });
    await rust.addStyleTag({ content: freezeAnimations });

    const results = [];
    for (const [tag, states] of Object.entries(samples)) {
      for (let index = 0; index < states.length; index += 1) {
        await renderState(upstream, tag, states[index]);
        await renderState(rust, tag, states[index]);
        for (const page of [upstream, rust]) {
          await page.locator(tag).evaluate((element) => {
            for (const node of element.shadowRoot?.querySelectorAll('*') ?? []) {
              if (node instanceof SVGElement || node instanceof HTMLElement) {
                node.style.animation = 'none';
                node.style.transition = 'none';
              }
            }
          });
        }
        const [up, rs] = await Promise.all([upstream.locator(tag).screenshot(), rust.locator(tag).screenshot()]);
        const comparison = await comparePngs(rust, up.toString('base64'), rs.toString('base64'));
        results.push({ tag, index, ...comparison });
        if (comparison.differentPixels > 0) {
          const dir = path.join(root, 'test-results', 'property-parity');
          fs.mkdirSync(dir, { recursive: true });
          fs.writeFileSync(path.join(dir, `${tag}-${index}-upstream.png`), up);
          fs.writeFileSync(path.join(dir, `${tag}-${index}-rust.png`), rs);
          fs.writeFileSync(path.join(dir, `${tag}-${index}-upstream.html`), await upstream.locator(tag).evaluate((element) => element.shadowRoot.innerHTML));
          fs.writeFileSync(path.join(dir, `${tag}-${index}-rust.html`), await rust.locator(tag).evaluate((element) => element.shadowRoot.innerHTML));
        }
      }
    }
    fs.mkdirSync(path.join(root, 'test-results'), { recursive: true });
    fs.writeFileSync(path.join(root, 'test-results', 'property-parity.json'), `${JSON.stringify(results, null, 2)}\n`);
    const mismatches = results.filter((result) => result.differentPixels > 0);
    assert.deepEqual(errors, []);
    assert.deepEqual(mismatches, []);
    console.log(`property parity passed for ${results.length} states across ${Object.keys(samples).length} components`);
  } finally {
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
