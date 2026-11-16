#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const contracts = JSON.parse(
  fs.readFileSync(path.join(root, 'reference/contracts/components.json'), 'utf8'),
);

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

async function renderShot(page, tag) {
  return page.evaluate(async (componentTag) => {
    document.body.replaceChildren();
    const element = document.createElement(componentTag);
    document.body.append(element);
    await element.updateComplete;
    return element.getBoundingClientRect().toJSON();
  }, tag);
}

async function comparePngs(page, leftBase64, rightBase64) {
  return page.evaluate(async ({ left, right }) => {
    const decode = async (base64) => {
      const bytes = Uint8Array.from(atob(base64), (character) => character.charCodeAt(0));
      const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }));
      const canvas = document.createElement('canvas');
      canvas.width = bitmap.width;
      canvas.height = bitmap.height;
      const context = canvas.getContext('2d', { willReadFrequently: true });
      context.drawImage(bitmap, 0, 0);
      return {
        width: bitmap.width,
        height: bitmap.height,
        data: context.getImageData(0, 0, bitmap.width, bitmap.height).data,
      };
    };
    const leftImage = await decode(left);
    const rightImage = await decode(right);
    if (
      leftImage.width !== rightImage.width ||
      leftImage.height !== rightImage.height
    ) {
      return {
        differentPixels: Infinity,
        totalPixels: Math.max(leftImage.width * leftImage.height, rightImage.width * rightImage.height),
        maxChannelDelta: 255,
        dimensions: [leftImage.width, leftImage.height, rightImage.width, rightImage.height],
      };
    }
    let differentPixels = 0;
    let maxChannelDelta = 0;
    for (let index = 0; index < leftImage.data.length; index += 4) {
      let different = false;
      for (let channel = 0; channel < 4; channel += 1) {
        const delta = Math.abs(leftImage.data[index + channel] - rightImage.data[index + channel]);
        maxChannelDelta = Math.max(maxChannelDelta, delta);
        if (delta > 0) different = true;
      }
      if (different) differentPixels += 1;
    }
    return {
      differentPixels,
      totalPixels: leftImage.width * leftImage.height,
      maxChannelDelta,
      dimensions: [leftImage.width, leftImage.height],
    };
  }, { left: leftBase64, right: rightBase64 });
}

async function main() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  const browser = await chromium.launch({ headless: true });

  try {
    const upstream = await browser.newPage({ viewport: { width: 1000, height: 1000 } });
    const rust = await browser.newPage({ viewport: { width: 1000, height: 1000 } });
    const errors = [];
    rust.on('pageerror', (error) => errors.push(error.message));
    await upstream.goto(`http://127.0.0.1:${port}/reference/upstream-render.html`);
    await upstream.waitForFunction(() => window.elements);
    await rust.goto(`http://127.0.0.1:${port}/tests/browser-fixture.html`);
    await rust.waitForFunction(() => window.wokwiReady === true);

    const results = [];
    for (const contract of contracts) {
      const tag = contract.tag;
      await renderShot(upstream, tag);
      await renderShot(rust, tag);
      const upstreamElement = upstream.locator(tag);
      const rustElement = rust.locator(tag);
      const [upstreamShot, rustShot] = await Promise.all([
        upstreamElement.screenshot(),
        rustElement.screenshot(),
      ]);
      const comparison = await comparePngs(
        rust,
        upstreamShot.toString('base64'),
        rustShot.toString('base64'),
      );
      results.push({ tag, ...comparison });
      if (comparison.differentPixels > 0) {
        const outputDir = path.join(root, 'test-results', 'visual-parity');
        fs.mkdirSync(outputDir, { recursive: true });
        fs.writeFileSync(path.join(outputDir, `${tag}-upstream.png`), upstreamShot);
        fs.writeFileSync(path.join(outputDir, `${tag}-rust.png`), rustShot);
        fs.writeFileSync(
          path.join(outputDir, `${tag}-upstream.html`),
          await upstreamElement.evaluate((element) => element.shadowRoot.innerHTML),
        );
        fs.writeFileSync(
          path.join(outputDir, `${tag}-rust.html`),
          await rustElement.evaluate((element) => element.shadowRoot.innerHTML),
        );
      }
    }

    fs.mkdirSync(path.join(root, 'test-results'), { recursive: true });
    fs.writeFileSync(
      path.join(root, 'test-results', 'visual-parity.json'),
      `${JSON.stringify(results, null, 2)}\n`,
    );
    const mismatches = results.filter((result) => result.differentPixels > 0);
    assert.deepEqual(errors, []);
    assert.deepEqual(mismatches, []);
    console.log(`visual parity passed for ${results.length} components`);
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
