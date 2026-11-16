#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const contracts = JSON.parse(fs.readFileSync(path.join(root, 'reference/contracts/components.json'), 'utf8'));
const pinScenarios = {
  'wokwi-7segment': [{ digits: 2, pins: 'extend' }],
  'wokwi-lcd1602': [{ pins: 'i2c' }],
  'wokwi-led': [{ flip: true }],
  'wokwi-led-ring': [{ pixels: 8, pixelSpacing: 1 }],
  'wokwi-membrane-keypad': [{ columns: '3' }],
  'wokwi-neopixel-matrix': [{ rows: 2, cols: 3, rowSpacing: 2, colSpacing: 0.5 }],
  'wokwi-slide-potentiometer': [{ travelLength: 45 }],
  'wokwi-stepper-motor': [{ size: 11 }],
};

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (typeof value === 'number') return Math.round(value * 1e9) / 1e9;
  if (value && typeof value === 'object') {
    const result = {};
    for (const key of Object.keys(value).sort()) {
      if (value[key] !== undefined) result[key] = canonical(value[key]);
    }
    return result;
  }
  return value;
}

function createServer() {
  return http.createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const filePath = path.resolve(root, `.${pathname}`);
    if (!filePath.startsWith(root)) { response.writeHead(403).end(); return; }
    fs.readFile(filePath, (error, data) => {
      if (error) { response.writeHead(404).end(); return; }
      const type = filePath.endsWith('.html') ? 'text/html' : filePath.endsWith('.js') ? 'text/javascript' : filePath.endsWith('.wasm') ? 'application/wasm' : 'application/octet-stream';
      response.writeHead(200, { 'content-type': type });
      response.end(data);
    });
  });
}

async function inspect(page, tag) {
  return page.evaluate(async (componentTag) => {
    document.body.replaceChildren();
    const element = document.createElement(componentTag);
    document.body.append(element);
    await element.updateComplete;
    const properties = {};
    for (const [name, value] of Object.entries(Object.fromEntries(
      Object.getOwnPropertyNames(Object.getPrototypeOf(element))
        .filter((name) => !['constructor', 'connectedCallback', 'attributeChangedCallback', 'updateComplete', 'pinInfo', 'requestUpdate', 'canvas', 'redraw', 'setPixel', 'reset', 'dial'].includes(name))
        .map((name) => [name, element[name]])
    ))) {
      if (name === 'imageData' || name === 'font') continue;
      properties[name] = value && typeof value === 'object' && 'data' in value ? { length: value.data.length } : value;
    }
    return {
      attributes: [...element.constructor.observedAttributes],
      pinInfo: element.pinInfo,
      properties,
    };
  }, tag);
}

async function inspectUpstream(page, tag) {
  return page.evaluate(async (componentTag) => {
    document.body.replaceChildren();
    const element = document.createElement(componentTag);
    document.body.append(element);
    await element.updateComplete;
    const properties = {};
    for (const name of element.constructor.elementProperties?.keys() ?? []) {
      if (name === 'imageData' || name === 'font') continue;
      const value = element[name];
      properties[name] = value && typeof value === 'object' && 'data' in value ? { length: value.data.length } : value;
    }
    return {
      attributes: [...element.constructor.observedAttributes],
      pinInfo: element.pinInfo,
      properties,
    };
  }, tag);
}

async function scenarioPinInfo(page, tag, properties) {
  return page.evaluate(async ({ componentTag, values }) => {
    document.body.replaceChildren();
    const element = document.createElement(componentTag);
    for (const [name, value] of Object.entries(values)) element[name] = value;
    document.body.append(element);
    await element.updateComplete;
    return element.pinInfo;
  }, { componentTag: tag, values: properties });
}

async function main() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  const browser = await chromium.launch({ headless: true });
  try {
    const upstream = await browser.newPage();
    const rust = await browser.newPage();
    await upstream.goto(`http://127.0.0.1:${port}/reference/upstream-render.html`);
    await upstream.waitForFunction(() => window.elements);
    await rust.goto(`http://127.0.0.1:${port}/tests/browser-fixture.html`);
    await rust.waitForFunction(() => window.wokwiReady === true);

    const mismatches = [];
    for (const contract of contracts) {
      const tag = contract.tag;
      const [expected, actual] = await Promise.all([inspectUpstream(upstream, tag), inspect(rust, tag)]);
      if (JSON.stringify(canonical(expected.attributes)) !== JSON.stringify(canonical(actual.attributes))) {
        mismatches.push({ tag, field: 'observedAttributes', expected: canonical(expected.attributes), actual: canonical(actual.attributes) });
      }
      if (JSON.stringify(canonical(expected.pinInfo)) !== JSON.stringify(canonical(actual.pinInfo))) {
        mismatches.push({ tag, field: 'pinInfo', expected: canonical(expected.pinInfo), actual: canonical(actual.pinInfo) });
      }
      if (JSON.stringify(canonical(expected.properties)) !== JSON.stringify(canonical(actual.properties))) {
        mismatches.push({ tag, field: 'properties', expected: canonical(expected.properties), actual: canonical(actual.properties) });
      }
    }
    for (const [tag, scenarios] of Object.entries(pinScenarios)) {
      for (let index = 0; index < scenarios.length; index += 1) {
        const expected = await scenarioPinInfo(upstream, tag, scenarios[index]);
        const actual = await scenarioPinInfo(rust, tag, scenarios[index]);
        if (JSON.stringify(canonical(expected)) !== JSON.stringify(canonical(actual))) {
          mismatches.push({ tag, field: `pinInfo scenario ${index}`, expected: canonical(expected), actual: canonical(actual) });
        }
      }
    }
    fs.writeFileSync(path.join(root, 'test-results', 'api-parity.json'), `${JSON.stringify(mismatches, null, 2)}\n`);
    assert.deepEqual(mismatches, []);
    console.log(`API parity passed for ${contracts.length} components`);
  } finally {
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
