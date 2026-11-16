import init, {
  component_contracts,
  default_state_json,
  dispatch_action,
  install_debug_hook,
  invoke_method,
  pin_info_json,
  render_component,
} from './pkg/wokwi_elements_rs.js';

await init();
install_debug_hook();

const contracts = JSON.parse(component_contracts());
const contractsByTag = new Map(contracts.map((contract) => [contract.tag, contract]));
const pinInfoProperties = {
  'wokwi-7segment': ['digits', 'pins'],
  'wokwi-lcd1602': ['pins'],
  'wokwi-lcd2004': ['pins'],
  'wokwi-led': ['flip'],
  'wokwi-led-ring': ['pixels', 'pixelSpacing'],
  'wokwi-membrane-keypad': ['columns'],
  'wokwi-neopixel-matrix': ['rows', 'cols', 'rowSpacing', 'colSpacing'],
  'wokwi-slide-potentiometer': ['travelLength'],
  'wokwi-stepper-motor': ['size'],
};

function valueFromAttribute(type, value) {
  if (value === null) {
    return type === 'Boolean' ? false : null;
  }
  switch (type) {
    case 'Boolean':
      return true;
    case 'Number':
      return Number(value);
    case 'Array':
    case 'Object':
      return JSON.parse(value);
    default:
      return value;
  }
}

function eventPayload(event) {
  const touch = event.touches?.[0] ?? event.changedTouches?.[0];
  const clientX = event.clientX ?? touch?.clientX;
  const clientY = event.clientY ?? touch?.clientY;
  const payload = {
    key: event.key,
    code: event.code,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
    altKey: event.altKey,
    button: event.button,
    detail: event.detail,
    checked: event.target?.checked,
    value: event.target?.value,
    clientX,
    clientY,
    dataBtn: event.target?.closest?.('[data-btn]')?.dataset.btn,
    dataKey: event.target?.closest?.('[data-key-name]')?.dataset.keyName,
  };
  const target = event.target instanceof Element ? event.target : null;
  const svg = target?.closest('svg');
  if (svg && clientX !== undefined && svg.getScreenCTM) {
    const matrix = svg.getScreenCTM();
    if (matrix) {
      const point = svg.createSVGPoint();
      point.x = clientX;
      point.y = clientY;
      const local = point.matrixTransform(matrix.inverse());
      payload.svgX = local.x;
      payload.svgY = local.y;
    }
  }
  return payload;
}

function morphNode(current, next) {
  if (current.nodeType !== next.nodeType || current.nodeName !== next.nodeName) {
    current.replaceWith(next.cloneNode(true));
    return;
  }

  if (current.nodeType === Node.TEXT_NODE || current.nodeType === Node.COMMENT_NODE) {
    if (current.nodeValue !== next.nodeValue) {
      current.nodeValue = next.nodeValue;
    }
    return;
  }

  if (current.nodeType !== Node.ELEMENT_NODE) return;

  for (const attribute of [...current.attributes]) {
    if (!next.hasAttribute(attribute.name)) {
      current.removeAttribute(attribute.name);
    }
  }
  for (const attribute of [...next.attributes]) {
    if (current.getAttribute(attribute.name) !== attribute.value) {
      current.setAttribute(attribute.name, attribute.value);
    }
  }

  const currentChildren = [...current.childNodes];
  const nextChildren = [...next.childNodes];
  for (let index = 0; index < nextChildren.length; index += 1) {
    const currentChild = currentChildren[index];
    if (currentChild) {
      morphNode(currentChild, nextChildren[index]);
    } else {
      current.append(nextChildren[index].cloneNode(true));
    }
  }
  for (let index = nextChildren.length; index < currentChildren.length; index += 1) {
    currentChildren[index].remove();
  }
}

function morphComponentRoot(shadowRoot, styleElement, html) {
  const template = document.createElement('template');
  template.innerHTML = html;
  const currentNodes = [...shadowRoot.childNodes].filter(
    (node) => node !== styleElement && node.nodeType !== Node.COMMENT_NODE,
  );
  const nextNodes = [...template.content.childNodes].filter(
    (node) => node.nodeType !== Node.COMMENT_NODE,
  );

  for (let index = 0; index < nextNodes.length; index += 1) {
    if (currentNodes[index]) {
      morphNode(currentNodes[index], nextNodes[index]);
    } else {
      shadowRoot.append(nextNodes[index].cloneNode(true));
    }
  }
  for (let index = nextNodes.length; index < currentNodes.length; index += 1) {
    currentNodes[index].remove();
  }
}

function defineElement(tag, contract) {
  const attributes = new Map();
  for (const [property, descriptor] of Object.entries(contract.properties)) {
    if (descriptor.attribute) {
      attributes.set(descriptor.attribute, { property, type: descriptor.type });
    }
  }

  class WokwiElement extends HTMLElement {
    static get observedAttributes() {
      return [...attributes.keys()];
    }

    static get tag() {
      return tag;
    }

    constructor() {
      super();
      this.attachShadow({ mode: 'open' });
      this.__state = JSON.parse(default_state_json(tag));
      if (tag === 'wokwi-ssd1306') {
        this.__imageData = new ImageData(128, 64);
      }
      this.__styleElement = document.createElement('style');
      this.shadowRoot.append(this.__styleElement);

      for (const eventName of [
        'pointerdown',
        'pointerup',
        'pointerleave',
        'pointercancel',
        'mousedown',
        'mouseleave',
        'mouseup',
        'mousemove',
        'touchstart',
        'touchend',
        'touchmove',
        'click',
        'input',
        'change',
        'blur',
        'keydown',
        'keyup',
      ]) {
        this.shadowRoot.addEventListener(
          eventName,
          (event) => this.__handleDomEvent(eventName, event),
          true,
        );
      }
    }

    connectedCallback() {
      this.__render();
      if (tag === 'wokwi-ili9341') {
        this.dispatchEvent(new CustomEvent('canvas-ready'));
      }
      if (tag === 'wokwi-ssd1306') {
        this.redraw?.();
      }
    }

    disconnectedCallback() {
      this.__stopArrowTimer();
      if (this.__animationFrame) {
        cancelAnimationFrame(this.__animationFrame);
        this.__animationFrame = null;
      }
    }

    attributeChangedCallback(name, oldValue, value) {
      if (oldValue === value) return;
      const attribute = attributes.get(name);
      if (!attribute) return;
      this.__setProperty(attribute.property, valueFromAttribute(attribute.type, value));
    }

    get updateComplete() {
      return Promise.resolve(true);
    }

    get pinInfo() {
      const pins = JSON.parse(pin_info_json(tag, JSON.stringify(this.__state)));
      return pins === null ? undefined : pins;
    }

    requestUpdate() {
      this.__render();
    }

    __setProperty(name, value) {
      const previous = this.__state[name];
      const changed = JSON.stringify(previous) !== JSON.stringify(value);
      if (!changed) return;
      this.__state[name] = value;
      this.__render();
      if (pinInfoProperties[tag]?.includes(name)) {
        this.dispatchEvent(new CustomEvent('pininfo-change'));
      }
    }

    __render() {
      const rendered = JSON.parse(render_component(tag, JSON.stringify(this.__state)));
      if (!rendered) {
        this.__styleElement.textContent = '';
        morphComponentRoot(
          this.shadowRoot,
          this.__styleElement,
          `<div data-wokwi-unimplemented="${tag}"></div>`,
        );
        return;
      }
      if (this.__styleElement.textContent !== rendered.styles) {
        this.__styleElement.textContent = rendered.styles;
      }
      morphComponentRoot(this.shadowRoot, this.__styleElement, rendered.html);
      this.__syncAnimation();
    }

    __handleDomEvent(eventName, event) {
      const target = event.target instanceof Element
        ? event.target.closest(`[data-on-${eventName}]`)
        : null;
      if (!target) return;

      const action = target.getAttribute(`data-on-${eventName}`);
      if (action.startsWith('arrowStart:')) {
        this.__startArrowTimer(action.slice('arrowStart:'.length));
        return;
      }
      if (action === 'arrowStop') {
        this.__stopArrowTimer();
        return;
      }
      this.__dispatchAction(action, eventPayload(event));
      if (tag === 'wokwi-slide-potentiometer' && action === 'down') {
        this.__startSlideDrag();
      }
      if (tag === 'wokwi-slide-potentiometer' && action === 'up') {
        this.__stopSlideDrag();
      }
    }

    __dispatchAction(action, payload) {
      const outcome = JSON.parse(
        dispatch_action(
          tag,
          JSON.stringify(this.__state),
          action,
          JSON.stringify(payload),
        ),
      );
      if (!outcome) return;

      this.__state = outcome.state;
      this.__render();
      for (const emitted of outcome.events) {
        const EventConstructor =
          emitted.type === 'CustomEvent'
            ? CustomEvent
            : emitted.type === 'InputEvent'
              ? InputEvent
              : Event;
        this.dispatchEvent(
          new EventConstructor(emitted.name, {
            bubbles: emitted.bubbles ?? false,
            composed: emitted.composed ?? false,
            detail: emitted.detail,
          }),
        );
      }
    }

    __startArrowTimer(direction) {
      this.__stopArrowTimer();
      this.__arrowTimer = setInterval(() => {
        this.__dispatchAction(direction, {});
      }, 300);
    }

    __stopArrowTimer() {
      if (this.__arrowTimer) {
        clearInterval(this.__arrowTimer);
        this.__arrowTimer = null;
      }
    }

    __syncAnimation() {
      if (tag !== 'wokwi-led-ring' && tag !== 'wokwi-neopixel-matrix') return;
      if (!this.__state.animation) {
        if (this.__animationFrame) cancelAnimationFrame(this.__animationFrame);
        this.__animationFrame = null;
        return;
      }
      if (this.__animationFrame) return;
      const tick = () => {
        if (!this.__state.animation) {
          this.__animationFrame = null;
          return;
        }
        this.__animationStep();
        this.__animationFrame = requestAnimationFrame(tick);
        this.__render();
      };
      this.__animationFrame = requestAnimationFrame(tick);
    }

    __animationStep() {
      const time = Date.now();
      const nextState = JSON.parse(
        invoke_method(
          tag,
          JSON.stringify(this.__state),
          'animationFrame',
          JSON.stringify([time]),
        ),
      );
      if (nextState) this.__state = nextState;
    }

    __startSlideDrag() {
      if (this.__slideDragging) return;
      this.__slideDragging = true;
      this.__slideMove = (event) => {
        const svg = this.shadowRoot.querySelector('svg');
        if (!svg?.getScreenCTM) return;
        const matrix = svg.getScreenCTM();
        if (!matrix) return;
        const point = svg.createSVGPoint();
        point.x = event.clientX;
        point.y = event.clientY;
        const local = point.matrixTransform(matrix.inverse());
        this.__dispatchAction('drag', { svgX: local.x, svgY: local.y });
      };
      this.__slideUp = () => {
        this.__dispatchAction('up', {});
        this.__stopSlideDrag();
      };
      window.addEventListener('mousemove', this.__slideMove);
      window.addEventListener('mouseup', this.__slideUp);
    }

    __stopSlideDrag() {
      if (!this.__slideDragging) return;
      this.__slideDragging = false;
      window.removeEventListener('mousemove', this.__slideMove);
      window.removeEventListener('mouseup', this.__slideUp);
    }
  }

  Object.defineProperty(WokwiElement, 'styles', {
    get() {
      return contractsByTag.get(tag)?.styles ?? null;
    },
  });

  for (const property of Object.keys(contract.properties)) {
    Object.defineProperty(WokwiElement.prototype, property, {
      configurable: true,
      enumerable: true,
      get() {
        return this.__state[property];
      },
      set(value) {
        this.__setProperty(property, value);
      },
    });
  }

  if (tag === 'wokwi-ili9341' || tag === 'wokwi-ssd1306') {
    Object.defineProperty(WokwiElement.prototype, 'canvas', {
      configurable: true,
      get() {
        return this.shadowRoot.querySelector('canvas');
      },
    });
  }

  if (tag === 'wokwi-ssd1306') {
    WokwiElement.prototype.redraw = function redraw() {
      const canvas = this.canvas;
      const context = canvas?.getContext('2d');
      if (context && this.__imageData) {
        context.putImageData(this.__imageData, 0, 0);
      }
    };
    Object.defineProperty(WokwiElement.prototype, 'imageData', {
      configurable: true,
      enumerable: true,
      get() {
        return this.__imageData;
      },
      set(value) {
        this.__imageData = value;
        this.redraw();
      },
    });
  }

  if (tag === 'wokwi-lcd1602' || tag === 'wokwi-lcd2004') {
    Object.defineProperty(WokwiElement.prototype, 'characters', {
      configurable: true,
      enumerable: true,
      get() {
        return Uint8Array.from(this.__state.characters ?? []);
      },
      set(value) {
        const characters = Array.from(value, (code) => Number(code) & 0xff);
        this.__state.characters = characters;
        this.__state.text = String.fromCharCode(...characters);
        this.__render();
      },
    });
    Object.defineProperty(WokwiElement.prototype, 'text', {
      configurable: true,
      enumerable: true,
      get() {
        return String.fromCharCode(...(this.__state.characters ?? []));
      },
      set(value) {
        const characters = Array.from(value, (character) => character.charCodeAt(0));
        this.__state.characters = characters;
        this.__state.text = value;
        this.__render();
      },
    });
    Object.defineProperty(WokwiElement.prototype, 'font', {
      configurable: true,
      enumerable: true,
      get() {
        return Uint8Array.from(this.__state.font ?? []);
      },
      set(value) {
        this.__state.font = Array.from(value);
        this.__render();
      },
    });
  }

  if (tag === 'wokwi-potentiometer') {
    WokwiElement.prototype.mapToMinMax = function mapToMinMax(value, min, max) {
      return value * (max - min) + min;
    };
    WokwiElement.prototype.percentFromMinMax = function percentFromMinMax(value, min, max) {
      return (value - min) / (max - min);
    };
  }

  if (tag === 'wokwi-led-ring') {
    WokwiElement.prototype.setPixel = function setPixel(pixel, color) {
      const nextState = JSON.parse(
        invoke_method(tag, JSON.stringify(this.__state), 'setPixel', JSON.stringify([pixel, color])),
      );
      if (nextState) {
        this.__state = nextState;
        this.__render();
      }
    };
    WokwiElement.prototype.reset = function resetPixels() {
      const nextState = JSON.parse(
        invoke_method(tag, JSON.stringify(this.__state), 'reset', '[]'),
      );
      if (nextState) {
        this.__state = nextState;
        this.__render();
      }
    };
  }

  if (tag === 'wokwi-neopixel-matrix') {
    WokwiElement.prototype.setPixel = function setPixel(row, col, color) {
      const nextState = JSON.parse(
        invoke_method(
          tag,
          JSON.stringify(this.__state),
          'setPixel',
          JSON.stringify([row, col, color]),
        ),
      );
      if (nextState) {
        this.__state = nextState;
        this.__render();
      }
    };
    WokwiElement.prototype.reset = function resetPixels() {
      const nextState = JSON.parse(
        invoke_method(tag, JSON.stringify(this.__state), 'reset', '[]'),
      );
      if (nextState) {
        this.__state = nextState;
        this.__render();
      }
    };
  }

  if (tag === 'wokwi-rotary-dialer') {
    WokwiElement.prototype.dial = function dial(digit) {
      const outcome = JSON.parse(
        dispatch_action(tag, JSON.stringify(this.__state), `dial:${digit}`, '{}'),
      );
      if (!outcome) return;
      this.__state = outcome.state;
      this.__render();
      for (const emitted of outcome.events) {
        this.dispatchEvent(new CustomEvent(emitted.name, { detail: emitted.detail }));
      }
      setTimeout(() => {
        this.dispatchEvent(new CustomEvent('dial', { detail: { digit: Number(digit) } }));
        setTimeout(() => {
          this.dispatchEvent(new CustomEvent('dial-end', { detail: { digit: Number(digit) } }));
          const nextState = JSON.parse(
            invoke_method(tag, JSON.stringify(this.__state), 'resetAnim', '[]'),
          );
          if (nextState) {
            this.__state = nextState;
            this.__render();
          }
        }, 1000);
      }, 0);
    };
  }

  customElements.define(tag, WokwiElement);
}

for (const contract of contracts) {
  defineElement(contract.tag, contract);
}

export default contracts;
