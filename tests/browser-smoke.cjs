#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');

function readTags() {
  const contracts = JSON.parse(
    fs.readFileSync(path.join(root, 'reference/contracts/components.json'), 'utf8'),
  );
  return contracts.map((component) => component.tag);
}

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
    if (!filePath.startsWith(root)) {
      response.writeHead(403).end();
      return;
    }
    fs.readFile(filePath, (error, data) => {
      if (error) {
        response.writeHead(404).end();
        return;
      }
      response.writeHead(200, { 'content-type': contentType(filePath) });
      response.end(data);
    });
  });
}

async function main() {
  const tags = readTags();
  const server = createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  const browser = await chromium.launch({ headless: true });

  try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(`http://127.0.0.1:${port}/tests/browser-fixture.html`);
    await page.waitForFunction(() => window.wokwiReady === true);

    const missing = await page.evaluate(
      (componentTags) => componentTags.filter((tag) => !customElements.get(tag)),
      tags,
    );
    assert.deepEqual(missing, []);
    assert.deepEqual(errors, []);

    const empty = await page.evaluate(async (componentTags) => {
      const result = [];
      for (const tag of componentTags) {
        const element = document.createElement(tag);
        document.body.append(element);
        await element.updateComplete;
        if (
          !element.shadowRoot?.innerHTML.trim() ||
          element.shadowRoot.querySelector('[data-wokwi-unimplemented]')
        ) {
          result.push(tag);
        }
        element.remove();
      }
      return result;
    }, tags);
    assert.deepEqual(empty, []);

    const led = await page.evaluate(async () => {
      const element = document.createElement('wokwi-led');
      document.body.append(element);
      await element.updateComplete;
      const off = element.shadowRoot.innerHTML;
      element.value = true;
      await element.updateComplete;
      return {
        off,
        on: element.shadowRoot.innerHTML,
        label: element.label,
        pins: element.pinInfo,
      };
    });

    assert.match(led.off, /class="light" style="display: none"/);
    assert.match(led.on, /fill="#ff8080"/);
    assert.match(led.on, /class="light" style="display: "/);
    assert.equal(led.label, '');
    assert.deepEqual(
      led.pins.map(({ name, x, y }) => ({ name, x, y })),
      [
        { name: 'A', x: 25, y: 42 },
        { name: 'C', x: 15, y: 42 },
      ],
    );

    const pushbutton = await page.evaluate(async () => {
      const element = document.createElement('wokwi-pushbutton');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('button-press', () => events.push('press'));
      element.addEventListener('button-release', () => events.push('release'));
      const button = element.shadowRoot.querySelector('button');
      button.dispatchEvent(
        new PointerEvent('pointerdown', { bubbles: true, composed: true }),
      );
      await element.updateComplete;
      const pressed = element.pressed;
      button.dispatchEvent(
        new PointerEvent('pointerup', { bubbles: true, composed: true }),
      );
      await element.updateComplete;
      return {
        events,
        pressed,
        released: element.pressed,
        html: element.shadowRoot.innerHTML,
      };
    });

    assert.deepEqual(pushbutton.events, ['press', 'release']);
    assert.equal(pushbutton.pressed, true);
    assert.equal(pushbutton.released, false);
    assert.match(pushbutton.html, /aria-label=" red pushbutton"/);

    const pushbutton6mm = await page.evaluate(async () => {
      const element = document.createElement('wokwi-pushbutton-6mm');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('button-press', () => events.push('press'));
      element.addEventListener('button-release', () => events.push('release'));
      const button = element.shadowRoot.querySelector('button');
      button.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
      await element.updateComplete;
      const pressed = element.pressed;
      button.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }));
      await element.updateComplete;
      return { events, pressed, released: element.pressed };
    });

    assert.deepEqual(pushbutton6mm, {
      events: ['press', 'release'],
      pressed: true,
      released: false,
    });

    const slideSwitch = await page.evaluate(async () => {
      const element = document.createElement('wokwi-slide-switch');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('input', (event) => events.push(event.detail));
      const svg = element.shadowRoot.querySelector('svg');
      svg.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }));
      await element.updateComplete;
      const first = element.value;
      svg.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }));
      await element.updateComplete;
      return { events, first, second: element.value };
    });

    assert.deepEqual(slideSwitch, { events: [1, 0], first: 1, second: 0 });

    const dipSwitch = await page.evaluate(async () => {
      const element = document.createElement('wokwi-dip-switch-8');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('switch-change', (event) => events.push(event.detail));
      const firstSwitch = element.shadowRoot.querySelector('g[fill="#917c6f"] rect');
      firstSwitch.dispatchEvent(new MouseEvent('click', { bubbles: true, composed: true }));
      await element.updateComplete;
      return {
        events,
        values: element.values,
        html: element.shadowRoot.innerHTML,
      };
    });

    assert.deepEqual(dipSwitch.events, [0]);
    assert.deepEqual(dipSwitch.values, [1, 0, 0, 0, 0, 0, 0, 0]);
    assert.match(dipSwitch.html, /xlink:href="#switch" x="0" y="-7.2"/);

    const potentiometer = await page.evaluate(async () => {
      const element = document.createElement('wokwi-potentiometer');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('input', () => events.push(element.value));
      const input = element.shadowRoot.querySelector('input');
      input.value = '1023';
      input.dispatchEvent(new Event('input', { bubbles: true }));
      await element.updateComplete;
      return {
        events,
        value: element.value,
        html: element.shadowRoot.innerHTML,
      };
    });

    assert.deepEqual(potentiometer.events, [1023]);
    assert.equal(potentiometer.value, 1023);
    assert.match(potentiometer.html, /--knob-angle: 135deg/);
    assert.match(potentiometer.html, /aria-valuenow="1023"/);

    const joystick = await page.evaluate(async () => {
      const element = document.createElement('wokwi-analog-joystick');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      for (const type of ['input', 'button-press', 'button-release']) {
        element.addEventListener(type, () => events.push(type));
      }
      const knob = element.shadowRoot.querySelector('#knob');
      const regions = element.shadowRoot.querySelectorAll('.controls .region');
      knob.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }));
      await element.updateComplete;
      const up = element.yValue;
      knob.dispatchEvent(new KeyboardEvent('keyup', { key: 'ArrowUp', bubbles: true }));
      await element.updateComplete;
      regions[0].dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await element.updateComplete;
      const left = element.xValue;
      regions[0].dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await element.updateComplete;
      element.shadowRoot
        .querySelector('.controls circle')
        .dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await element.updateComplete;
      const pressed = element.pressed;
      element.shadowRoot
        .querySelector('.controls circle')
        .dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await element.updateComplete;
      return {
        events,
        up,
        left,
        pressed,
        released: element.pressed,
        transform: element.shadowRoot.querySelector('#knob').getAttribute('transform'),
      };
    });

    assert.equal(joystick.up, 1);
    assert.equal(joystick.left, 1);
    assert.equal(joystick.pressed, true);
    assert.equal(joystick.released, false);
    assert.equal(joystick.transform, 'translate(0, 0)');
    assert.deepEqual(joystick.events, [
      'input',
      'input',
      'input',
      'input',
      'button-press',
      'button-release',
    ]);

    const ledRing = await page.evaluate(async () => {
      const element = document.createElement('wokwi-led-ring');
      element.pixels = 8;
      document.body.append(element);
      await element.updateComplete;
      element.setPixel(0, { r: 1, g: 0, b: 0 });
      await element.updateComplete;
      const pixels = [...element.shadowRoot.querySelectorAll('rect.pixel')];
      return {
        count: pixels.length,
        firstFill: pixels[0].style.fill,
        pins: element.pinInfo.map((pin) => pin.name),
      };
    });

    assert.deepEqual(ledRing, {
      count: 8,
      firstFill: 'rgb(255, 0, 0)',
      pins: ['GND', 'VCC', 'DIN', 'DOUT'],
    });

    const matrix = await page.evaluate(async () => {
      const element = document.createElement('wokwi-neopixel-matrix');
      element.rows = 2;
      element.cols = 3;
      document.body.append(element);
      await element.updateComplete;
      element.setPixel(1, 2, { r: 1, g: 0, b: 0 });
      await element.updateComplete;
      const pixels = [...element.shadowRoot.querySelectorAll('g.pixel')];
      const target = [...pixels[5].querySelectorAll('ellipse')];
      const result = {
        count: pixels.length,
        opacities: target.slice(0, 3).map((pixel) => pixel.style.opacity),
        glowOpacity: target[3].style.opacity,
        glowFill: target[3].style.fill,
      };
      element.reset();
      await element.updateComplete;
      result.afterReset = [...element.shadowRoot.querySelectorAll('g.pixel')][5]
        .querySelectorAll('ellipse')[3].style.opacity;
      return result;
    });

    assert.deepEqual(matrix, {
      count: 6,
      opacities: ['1', '0', '0'],
      glowOpacity: '1',
      glowFill: 'rgb(255, 0, 0)',
      afterReset: '0',
    });

    const encoder = await page.evaluate(async () => {
      const element = document.createElement('wokwi-ky-040');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      for (const type of ['rotate-cw', 'rotate-ccw', 'button-press', 'button-release']) {
        element.addEventListener(type, () => events.push(type));
      }
      const arrows = element.shadowRoot.querySelectorAll('.arrow-container');
      arrows[1].dispatchEvent(new MouseEvent('click', { bubbles: true }));
      await element.updateComplete;
      const angle = element.angle;
      const handle = element.shadowRoot.querySelector('.handle');
      handle.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await element.updateComplete;
      const pressed = element.pressed;
      handle.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await element.updateComplete;
      return { events, angle, pressed, released: element.pressed };
    });

    assert.deepEqual(encoder, {
      events: ['rotate-cw', 'button-press', 'button-release'],
      angle: 18,
      pressed: true,
      released: false,
    });

    const irRemote = await page.evaluate(async () => {
      const element = document.createElement('wokwi-ir-remote');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      element.addEventListener('button-press', (event) => events.push(['press', event.detail]));
      element.addEventListener('button-release', (event) => events.push(['release', event.detail]));
      const button = element.shadowRoot.querySelector('use[data-btn="power"]');
      button.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await element.updateComplete;
      button.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await element.updateComplete;
      return { events, active: element.shadowRoot.querySelector('use.active')?.dataset.btn ?? null };
    });

    assert.deepEqual(irRemote, {
      events: [
        ['press', { key: 'power', irCode: 162 }],
        ['release', { key: 'power', irCode: 162 }],
      ],
      active: null,
    });

    const dialer = await page.evaluate(async () => {
      const element = document.createElement('wokwi-rotary-dialer');
      document.body.append(element);
      await element.updateComplete;
      const events = [];
      for (const type of ['dial-start', 'dial', 'dial-end']) {
        element.addEventListener(type, (event) => events.push([type, event.detail.digit]));
      }
      element.dial(5);
      await new Promise((resolve) => setTimeout(resolve, 1150));
      return {
        events,
        slots: element.shadowRoot.querySelector('#slots').getAttribute('style'),
      };
    });

    assert.deepEqual(dialer.events, [
      ['dial-start', 5],
      ['dial', 5],
      ['dial-end', 5],
    ]);
    assert.equal(dialer.slots, '');

    const displays = await page.evaluate(async () => {
      const ili = document.createElement('wokwi-ili9341');
      let ready = 0;
      ili.addEventListener('canvas-ready', () => { ready += 1; });
      document.body.append(ili);
      await ili.updateComplete;
      const iliCanvas = ili.canvas;
      ili.flipHorizontal = true;
      await ili.updateComplete;
      const flipStyle = ili.shadowRoot.querySelector('canvas').style.transform;

      const oled = document.createElement('wokwi-ssd1306');
      document.body.append(oled);
      await oled.updateComplete;
      const context = oled.canvas.getContext('2d');
      const image = context.createImageData(128, 64);
      image.data.set([0, 255, 0, 255], 0);
      oled.imageData = image;
      oled.redraw();
      const pixel = [...context.getImageData(0, 0, 1, 1).data];
      return {
        ready,
        iliCanvas: [iliCanvas.width, iliCanvas.height],
        flipStyle,
        oledCanvas: [oled.canvas.width, oled.canvas.height],
        pixel,
      };
    });

    assert.deepEqual(displays, {
      ready: 1,
      iliCanvas: [240, 320],
      flipStyle: 'scaleX(-1) scaleY(1)',
      oledCanvas: [128, 64],
      pixel: [0, 255, 0, 255],
    });

    const displayApi = await page.evaluate(async () => {
      const lcd = document.createElement('wokwi-lcd1602');
      document.body.append(lcd);
      await lcd.updateComplete;
      lcd.text = 'Hi';
      const fromText = {
        text: lcd.text,
        characters: lcd.characters instanceof Uint8Array,
        codes: [...lcd.characters].slice(0, 2),
      };
      lcd.characters = new Uint8Array([65, 66]);
      const fromCharacters = lcd.text;
      const oled = document.createElement('wokwi-ssd1306');
      document.body.append(oled);
      await oled.updateComplete;
      return {
        fromText,
        fromCharacters,
        imageData: oled.imageData instanceof ImageData,
        imageSize: [oled.imageData.width, oled.imageData.height],
        font: lcd.font instanceof Uint8Array,
      };
    });

    assert.deepEqual(displayApi, {
      fromText: { text: 'Hi', characters: true, codes: [72, 105] },
      fromCharacters: 'AB',
      imageData: true,
      imageSize: [128, 64],
      font: true,
    });

    const inputDetails = await page.evaluate(async () => {
      const keypad = document.createElement('wokwi-membrane-keypad');
      document.body.append(keypad);
      await keypad.updateComplete;
      const keypadEvents = [];
      keypad.addEventListener('button-press', (event) => keypadEvents.push(['press', event.detail]));
      keypad.addEventListener('button-release', (event) => keypadEvents.push(['release', event.detail]));
      const key = keypad.shadowRoot.querySelector('[data-key-name="1"]');
      key.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await keypad.updateComplete;
      const pressed = keypad.shadowRoot.querySelector('[data-key-name="1"]').classList.contains('pressed');
      keypad.shadowRoot.querySelector('[data-key-name="1"]').dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await keypad.updateComplete;

      const board = document.createElement('wokwi-arduino-uno');
      document.body.append(board);
      await board.updateComplete;
      const boardEvents = [];
      board.addEventListener('button-press', (event) => boardEvents.push(['press', event.detail]));
      board.addEventListener('button-release', (event) => boardEvents.push(['release', event.detail]));
      const reset = board.shadowRoot.querySelector('#reset-button');
      reset.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await board.updateComplete;
      const resetPressed = board.resetPressed;
      board.shadowRoot.querySelector('#reset-button').dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await board.updateComplete;
      return {
        keypadEvents,
        pressed,
        boardEvents,
        resetPressed,
        resetReleased: board.resetPressed,
      };
    });

    assert.deepEqual(inputDetails, {
      keypadEvents: [
        ['press', { key: '1', row: 0, column: 0 }],
        ['release', { key: '1', row: 0, column: 0 }],
      ],
      pressed: true,
      boardEvents: [
        ['press', 'reset'],
        ['release', 'reset'],
      ],
      resetPressed: true,
      resetReleased: false,
    });

    const heldEncoder = await page.evaluate(async () => {
      const element = document.createElement('wokwi-ky-040');
      document.body.append(element);
      await element.updateComplete;
      const arrow = element.shadowRoot.querySelectorAll('.arrow-container')[1];
      arrow.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      await new Promise((resolve) => setTimeout(resolve, 360));
      arrow.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
      await element.updateComplete;
      return element.angle;
    });

    assert.equal(heldEncoder, 18);

    const shiftedKeypad = await page.evaluate(async () => {
      const element = document.createElement('wokwi-membrane-keypad');
      document.body.append(element);
      await element.updateComplete;
      const releases = [];
      element.addEventListener('button-release', (event) => releases.push(event.detail.key));
      for (const key of ['1', 'A']) {
        element.shadowRoot
          .querySelector(`[data-key-name="${key}"]`)
          .dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
      }
      await element.updateComplete;
      element.shadowRoot.querySelector('svg').dispatchEvent(
        new KeyboardEvent('keyup', { key: 'Shift', bubbles: true }),
      );
      await element.updateComplete;
      return {
        releases,
        pressed: element.shadowRoot.querySelectorAll('.pressed').length,
      };
    });

    assert.deepEqual(shiftedKeypad, { releases: ['1', 'A'], pressed: 0 });

    const animations = await page.evaluate(async () => {
      const ring = document.createElement('wokwi-led-ring');
      ring.pixels = 4;
      ring.animation = true;
      document.body.append(ring);
      await ring.updateComplete;
      await new Promise((resolve) => setTimeout(resolve, 120));
      const ringFill = ring.shadowRoot.querySelector('rect.pixel').style.fill;
      ring.animation = false;

      const matrix = document.createElement('wokwi-neopixel-matrix');
      matrix.rows = 2;
      matrix.cols = 2;
      matrix.animation = true;
      document.body.append(matrix);
      await matrix.updateComplete;
      await new Promise((resolve) => setTimeout(resolve, 120));
      const matrixOpacity = [...matrix.shadowRoot.querySelectorAll('g.pixel ellipse')][3].style.opacity;
      matrix.animation = false;
      return { ringFill, matrixOpacity };
    });

    assert.match(animations.ringFill, /^rgb\(/);
    assert.notEqual(animations.matrixOpacity, '');

    const slideDrag = await page.evaluate(async () => {
      const element = document.createElement('wokwi-slide-potentiometer');
      element.travelLength = 30;
      document.body.append(element);
      await element.updateComplete;
      const svg = element.shadowRoot.querySelector('svg');
      const point = svg.createSVGPoint();
      point.x = 22.5;
      point.y = 10;
      const screen = point.matrixTransform(svg.getScreenCTM());
      element.shadowRoot
        .querySelector('#tip')
        .dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: screen.x, clientY: screen.y }));
      window.dispatchEvent(new MouseEvent('mousemove', { clientX: screen.x, clientY: screen.y }));
      window.dispatchEvent(new MouseEvent('mouseup', { clientX: screen.x, clientY: screen.y }));
      await element.updateComplete;
      return element.value;
    });

    assert.equal(slideDrag, 50);

    const pinInfoEvents = await page.evaluate(async () => {
      const counts = {};
      for (const [tag, mutations] of Object.entries({
        'wokwi-7segment': [['digits', 2], ['digits', 2], ['pins', 'extend']],
        'wokwi-led': [['flip', true]],
        'wokwi-membrane-keypad': [['columns', '3']],
        'wokwi-stepper-motor': [['size', 11]],
      })) {
        const element = document.createElement(tag);
        document.body.append(element);
        await element.updateComplete;
        let count = 0;
        element.addEventListener('pininfo-change', () => { count += 1; });
        for (const [name, value] of mutations) element[name] = value;
        counts[tag] = count;
      }
      return counts;
    });

    assert.deepEqual(pinInfoEvents, {
      'wokwi-7segment': 2,
      'wokwi-led': 1,
      'wokwi-membrane-keypad': 1,
      'wokwi-stepper-motor': 1,
    });

    const keyboardControls = await page.evaluate(async () => {
      const dip = document.createElement('wokwi-dip-switch-8');
      document.body.append(dip);
      await dip.updateComplete;
      const dipEvents = [];
      dip.addEventListener('switch-change', (event) => dipEvents.push(event.detail));
      dip.shadowRoot.querySelector('svg').dispatchEvent(
        new KeyboardEvent('keydown', { key: '3', bubbles: true }),
      );
      await dip.updateComplete;

      const keypad = document.createElement('wokwi-membrane-keypad');
      document.body.append(keypad);
      await keypad.updateComplete;
      const keypadEvents = [];
      keypad.addEventListener('button-press', (event) => keypadEvents.push(['press', event.detail.key]));
      keypad.addEventListener('button-release', (event) => keypadEvents.push(['release', event.detail.key]));
      const key = keypad.shadowRoot.querySelector('[data-key-name="A"]');
      key.dispatchEvent(new KeyboardEvent('keydown', { key: ' ', bubbles: true }));
      await keypad.updateComplete;
      keypad.shadowRoot.querySelector('[data-key-name="A"]').dispatchEvent(
        new KeyboardEvent('keyup', { key: ' ', bubbles: true }),
      );
      await keypad.updateComplete;
      return { dipValues: dip.values, dipEvents, keypadEvents };
    });

    assert.deepEqual(keyboardControls, {
      dipValues: [0, 0, 1, 0, 0, 0, 0, 0],
      dipEvents: [2],
      keypadEvents: [['press', 'A'], ['release', 'A']],
    });

    const potentiometerMath = await page.evaluate(() => {
      const element = document.createElement('wokwi-potentiometer');
      return {
        map: element.mapToMinMax(0.5, 0, 1023),
        percent: element.percentFromMinMax(511.5, 0, 1023),
      };
    });
    assert.deepEqual(potentiometerMath, { map: 511.5, percent: 0.5 });

    const remoteKeyboard = await page.evaluate(async () => {
      const remote = document.createElement('wokwi-ir-remote');
      document.body.append(remote);
      await remote.updateComplete;
      const remoteEvents = [];
      remote.addEventListener('button-press', (event) => remoteEvents.push(['press', event.detail]));
      remote.addEventListener('button-release', (event) => remoteEvents.push(['release', event.detail]));
      const svg = remote.shadowRoot.querySelector('svg');
      svg.dispatchEvent(new KeyboardEvent('keydown', { key: 'o', bubbles: true }));
      svg.dispatchEvent(new KeyboardEvent('keyup', { key: 'o', bubbles: true }));
      return remoteEvents;
    });

    assert.deepEqual(remoteKeyboard, [
      ['press', { key: 'power', irCode: 162 }],
      ['release', { key: 'power', irCode: 162 }],
    ]);

    console.log('browser smoke passed');
  } finally {
    await browser.close();
    await new Promise((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
