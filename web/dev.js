import contracts from './index.js';

const EVENT_NAMES = [
  'button-press', 'button-release', 'input', 'switch-change', 'pininfo-change',
  'canvas-ready', 'rotate-cw', 'rotate-ccw', 'dial-start', 'dial', 'dial-end',
];

const state = {
  selected: null,
  selectedStory: null,
  element: null,
  events: [],
};

const ui = {
  catalog: document.querySelector('#catalog'),
  stories: document.querySelector('#stories'),
  search: document.querySelector('#search'),
  stage: document.querySelector('#stage'),
  preview: document.querySelector('#preview'),
  currentTag: document.querySelector('#current-tag'),
  controls: document.querySelector('#controls'),
  pins: document.querySelector('#pins'),
  events: document.querySelector('#events'),
  snippet: document.querySelector('#snippet'),
  grid: document.querySelector('#grid'),
  zoom: document.querySelector('#zoom'),
  reset: document.querySelector('#reset'),
  theme: document.querySelector('#theme'),
  clearEvents: document.querySelector('#clear-events'),
};

const byTag = new Map(contracts.map((contract) => [contract.tag, contract]));

const storyPresets = {
  'wokwi-led': [
    { name: 'Lit', state: { value: true, label: 'ON' } },
    { name: 'Dim', state: { value: true, brightness: 0.25 } },
    { name: 'Green', state: { value: true, color: 'green' } },
    { name: 'Flipped', state: { value: true, flip: true } },
  ],
  'wokwi-7segment': [
    { name: 'Two digits', state: { digits: 2, values: [1,1,1,1,1,1,0,0,1,1,0,0,1,1,1,0] } },
    { name: 'Clock colon', state: { digits: 4, colon: true, colonValue: true, values: Array(32).fill(1) } },
  ],
  'wokwi-rgb-led': [
    { name: 'Red', state: { ledRed: 1, ledGreen: 0, ledBlue: 0 } },
    { name: 'White', state: { ledRed: 1, ledGreen: 1, ledBlue: 1 } },
    { name: 'Mixed', state: { ledRed: 0.5, ledGreen: 1, ledBlue: 0.25 } },
  ],
  'wokwi-neopixel': [
    { name: 'Red', state: { r: 1, g: 0, b: 0 } },
    { name: 'Blue', state: { r: 0, g: 0, b: 1 } },
    { name: 'White', state: { r: 1, g: 1, b: 1 } },
  ],
  'wokwi-led-bar-graph': [
    { name: 'Half lit', state: { values: [1,1,1,1,1,0,0,0,0,0] } },
    { name: 'GYR full', state: { color: 'GYR', values: Array(10).fill(1) } },
  ],
  'wokwi-pushbutton': [
    { name: 'Pressed', state: { pressed: true } },
    { name: 'Labelled', state: { label: 'START', color: 'green' } },
    { name: 'X-ray', state: { xray: true } },
  ],
  'wokwi-pushbutton-6mm': [
    { name: 'Pressed', state: { pressed: true } },
    { name: 'Labelled', state: { label: '6mm', color: 'blue' } },
  ],
  'wokwi-dip-switch-8': [
    { name: 'Alternating', state: { values: [1,0,1,0,1,0,1,0] } },
    { name: 'All on', state: { values: Array(8).fill(1) } },
  ],
  'wokwi-slide-switch': [{ name: 'On', state: { value: 1 } }],
  'wokwi-potentiometer': [
    { name: 'Minimum', state: { value: 0 } },
    { name: 'Middle', state: { value: 511.5 } },
    { name: 'Maximum', state: { value: 1023 } },
  ],
  'wokwi-slide-potentiometer': [
    { name: 'Minimum', state: { value: 0 } },
    { name: 'Middle', state: { value: 50 } },
    { name: 'Maximum', state: { value: 100 } },
    { name: 'Long travel', state: { travelLength: 45, value: 50 } },
  ],
  'wokwi-servo': [
    { name: 'Left', state: { angle: -90 } },
    { name: 'Right', state: { angle: 90 } },
    { name: 'Cross horn', state: { horn: 'cross', hornColor: '#ff5500', angle: 45 } },
    { name: 'Double horn', state: { horn: 'double', angle: -45 } },
  ],
  'wokwi-stepper-motor': [
    { name: 'NEMA 11', state: { size: 11, angle: 90, arrow: 'red', value: '42', units: 'steps' } },
    { name: 'NEMA 34', state: { size: 34, angle: -90 } },
  ],
  'wokwi-biaxial-stepper': [
    { name: 'Arrow + ornate', state: { outerHandShape: 'arrow', outerHandAngle: 90, outerHandColor: 'red', innerHandShape: 'ornate', innerHandAngle: -30, innerHandColor: 'blue' } },
    { name: 'Plain hands', state: { outerHandColor: 'gold', innerHandColor: 'silver' } },
  ],
  'wokwi-hx711': [
    { name: '50 kg', state: { type: '50kg' } },
    { name: '5 kg', state: { type: '5kg' } },
    { name: 'Gauge', state: { type: 'gauge' } },
  ],
  'wokwi-lcd1602': [
    { name: 'Hello', state: { text: 'Hello, Wokwi!' } },
    { name: 'I2C blue', state: { text: 'I2C display', pins: 'i2c', background: 'blue' } },
    { name: 'Screen only', state: { text: 'Screen only', screenOnly: true } },
    { name: 'Cursor', state: { text: 'Cursor', cursor: true, cursorX: 3, cursorY: 1 } },
  ],
  'wokwi-lcd2004': [
    { name: 'Text', state: { text: 'Boardless 20x4 display' } },
    { name: 'I2C', state: { text: 'I2C 20x4', pins: 'i2c' } },
  ],
  'wokwi-membrane-keypad': [
    { name: '3 columns', state: { columns: '3' } },
    { name: 'Connector', state: { connector: true } },
    { name: 'Calculator keys', state: { keys: ['1','2','3','+','4','5','6','-','7','8','9','*','C','0','=','/'] } },
  ],
  'wokwi-analog-joystick': [
    { name: 'Up left', state: { xValue: 1, yValue: 1 } },
    { name: 'Pressed', state: { pressed: true } },
  ],
  'wokwi-ky-040': [
    { name: 'Rotated', state: { angle: 90 } },
    { name: 'Pressed', state: { pressed: true } },
  ],
  'wokwi-neopixel-matrix': [
    { name: '2 × 3', state: { rows: 2, cols: 3 } },
    { name: 'Blurred', state: { blurLight: true } },
  ],
  'wokwi-led-ring': [
    { name: '8 pixels', state: { pixels: 8 } },
    { name: '16 pixels', state: { pixels: 16 } },
  ],
  'wokwi-ssd1306': [{ name: 'Default', state: {} }],
  'wokwi-ili9341': [
    { name: 'Flip horizontal', state: { flipHorizontal: true } },
    { name: 'Flip vertical', state: { flipVertical: true } },
  ],
  'wokwi-big-sound-sensor': [{ name: 'Both LEDs', state: { led1: true, led2: true } }],
  'wokwi-flame-sensor': [{ name: 'Both LEDs', state: { ledPower: true, ledSignal: true } }],
  'wokwi-gas-sensor': [{ name: 'Both LEDs', state: { ledPower: true, ledD0: true } }],
  'wokwi-photoresistor-sensor': [{ name: 'Both LEDs', state: { ledPower: true, ledDO: true } }],
  'wokwi-small-sound-sensor': [{ name: 'Both LEDs', state: { ledPower: true, ledSignal: true } }],
  'wokwi-mpu6050': [{ name: 'LED on', state: { led1: true } }],
  'wokwi-arduino-uno': [{ name: 'LEDs + reset', state: { led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true } }],
  'wokwi-arduino-nano': [{ name: 'LEDs + reset', state: { led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true } }],
  'wokwi-arduino-mega': [{ name: 'LEDs + reset', state: { led13: true, ledRX: true, ledTX: true, ledPower: true, resetPressed: true } }],
  'wokwi-franzininho': [{ name: 'LEDs + reset', state: { led1: true, ledPower: true, resetPressed: true } }],
  'wokwi-esp32-devkit-v1': [{ name: 'LEDs on', state: { led1: true, ledPower: true } }],
  'wokwi-nano-rp2040-connect': [{ name: 'RGB + power', state: { ledRed: 1, ledGreen: 0.5, ledBlue: 0, ledPower: true, ledBuiltIn: true } }],
};

function storiesFor(tag) {
  return [{ name: 'Default', state: {} }, ...(storyPresets[tag] ?? [])];
}

function formatValue(value) {
  if (value instanceof Uint8Array || value instanceof Uint8ClampedArray) {
    return `Uint8${value instanceof Uint8ClampedArray ? 'Clamped' : ''}Array(${value.length})`;
  }
  if (value && typeof value === 'object' && 'data' in value) return `ImageData(${value.width}×${value.height})`;
  if (Array.isArray(value)) return `Array(${value.length})`;
  if (typeof value === 'string' && value.length > 40) return `"${value.slice(0, 37)}…"`;
  return JSON.stringify(value);
}

function renderCatalog(filter = '') {
  ui.catalog.replaceChildren();
  const query = filter.trim().toLowerCase();
  for (const contract of contracts) {
    if (query && !contract.tag.includes(query) && !contract.className.toLowerCase().includes(query)) continue;
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = contract.tag;
    button.className = contract.tag === state.selected ? 'active' : '';
    button.addEventListener('click', () => select(contract.tag));
    ui.catalog.append(button);
  }
}

function renderStories() {
  ui.stories.replaceChildren();
  if (!state.selected) return;
  const stories = storiesFor(state.selected);
  if (state.selectedStory === 'Custom' && !stories.some((story) => story.name === 'Custom')) {
    stories.push({ name: 'Custom', state: {} });
  }
  for (const story of stories) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = story.name;
    button.className = story.name === state.selectedStory ? 'active' : '';
    button.addEventListener('click', () => {
      if (story.name !== 'Custom') select(state.selected, story.name, story.state);
    });
    ui.stories.append(button);
  }
}

function appendEvent(name, event) {
  const entry = event?.detail === undefined ? name : `${name} ${JSON.stringify(event.detail)}`;
  state.events.unshift(entry);
  state.events = state.events.slice(0, 80);
  ui.events.textContent = state.events.join('\n') || '—';
}

function attachListeners(element) {
  for (const name of EVENT_NAMES) {
    element.addEventListener(name, (event) => appendEvent(name, event));
  }
}

function buildControl(element, name, descriptor) {
  const defaultValue = descriptor.default?.kind === 'value' ? descriptor.default.value : undefined;
  const booleanLike = descriptor.type === 'Boolean' || typeof defaultValue === 'boolean';
  const numberLike = descriptor.type === 'Number' || typeof defaultValue === 'number';
  const wrap = document.createElement('div');
  wrap.className = 'control';
  const label = document.createElement('label');
  const title = document.createElement('span');
  title.textContent = name;
  const meta = document.createElement('span');
  meta.textContent = descriptor.type || 'value';
  label.append(title, meta);
  wrap.append(label);

  const apply = (value) => {
    try {
      element[name] = value;
      state.selectedStory = 'Custom';
      renderStories();
      updateInspector();
    } catch (error) {
      appendEvent(`error:${name}`, { detail: String(error) });
    }
  };

  if (booleanLike) {
    const input = document.createElement('input');
    input.type = 'checkbox';
    input.checked = Boolean(element[name]);
    input.addEventListener('change', () => apply(input.checked));
    wrap.append(input);
    return wrap;
  }

  if (numberLike) {
    const input = document.createElement('input');
    input.type = 'number';
    input.step = 'any';
    input.value = String(element[name] ?? 0);
    input.addEventListener('input', () => {
      if (input.value !== '' && Number.isFinite(Number(input.value))) apply(Number(input.value));
    });
    wrap.append(input);
    return wrap;
  }

  if (descriptor.type === 'String' && name !== 'font') {
    const input = document.createElement('input');
    input.type = 'text';
    input.value = element[name] ?? '';
    input.addEventListener('input', () => apply(input.value));
    wrap.append(input);
    return wrap;
  }

  if (name === 'imageData') {
    const note = document.createElement('div');
    note.className = 'hint';
    note.textContent = 'Use the console or a custom test to supply ImageData.';
    wrap.append(note);
    return wrap;
  }

  const textarea = document.createElement('textarea');
  const current = element[name];
  if (current instanceof Uint8Array || current instanceof Uint8ClampedArray) {
    textarea.value = JSON.stringify(Array.from(current));
  } else if (Array.isArray(current)) {
    textarea.value = JSON.stringify(current);
  } else if (current && typeof current === 'object') {
    textarea.value = JSON.stringify(current);
  } else {
    textarea.value = JSON.stringify(element[name] ?? null);
  }
  textarea.addEventListener('change', () => {
    try {
      const parsed = JSON.parse(textarea.value);
      apply(Array.isArray(parsed) && (name === 'characters' || name === 'font') ? Uint8Array.from(parsed) : parsed);
    } catch {
      appendEvent(`error:${name}`, { detail: 'Invalid JSON' });
    }
  });
  wrap.append(textarea);
  return wrap;
}

function renderControls() {
  ui.controls.replaceChildren();
  if (!state.element) return;
  const contract = byTag.get(state.selected);
  for (const [name, descriptor] of Object.entries(contract.properties)) {
    ui.controls.append(buildControl(state.element, name, descriptor));
  }
  if (!Object.keys(contract.properties).length) {
    const empty = document.createElement('p');
    empty.className = 'hint';
    empty.textContent = 'This static component has no writable public properties.';
    ui.controls.append(empty);
  }
}

function updateInspector() {
  if (!state.element) return;
  const pins = state.element.pinInfo;
  ui.pins.textContent = pins === undefined ? 'undefined' : JSON.stringify(pins, null, 2);
  const contract = byTag.get(state.selected);
  const lines = [`const element = document.createElement('${state.selected}');`];
  for (const [name] of Object.entries(contract.properties)) {
    const value = state.element[name];
    if (value === undefined || name === 'imageData' || name === 'font') continue;
    if (value instanceof Uint8Array || value instanceof Uint8ClampedArray) {
      lines.push(`element.${name} = Uint8Array.from(${JSON.stringify(Array.from(value))});`);
    } else if (Array.isArray(value) || (value && typeof value === 'object')) {
      lines.push(`element.${name} = ${JSON.stringify(value)};`);
    } else {
      lines.push(`element.${name} = ${JSON.stringify(value)};`);
    }
  }
  lines.push('document.body.append(element);');
  ui.snippet.textContent = lines.join('\n');
}

function select(tag, storyName = 'Default', storyState = {}) {
  state.selected = tag;
  state.selectedStory = storyName;
  state.events = [];
  ui.events.textContent = '—';
  ui.currentTag.textContent = tag;
  ui.preview.className = '';
  ui.preview.replaceChildren();
  const element = document.createElement(tag);
  for (const [name, value] of Object.entries(storyState)) {
    element[name] = value;
  }
  state.element = element;
  attachListeners(element);
  ui.preview.append(element);
  renderCatalog(ui.search.value);
  renderStories();
  ui.controls.replaceChildren();
  requestAnimationFrame(() => {
    renderControls();
    updateInspector();
  });
}

ui.search.addEventListener('input', () => renderCatalog(ui.search.value));
ui.grid.addEventListener('change', () => ui.stage.classList.toggle('plain', !ui.grid.checked));
ui.zoom.addEventListener('change', () => {
  ui.preview.style.transform = `scale(${ui.zoom.value})`;
});
ui.theme.addEventListener('click', () => document.body.classList.toggle('dark'));
ui.clearEvents.addEventListener('click', () => {
  state.events = [];
  ui.events.textContent = '—';
});
ui.reset.addEventListener('click', () => {
  if (state.selected) select(state.selected);
});

renderCatalog();
