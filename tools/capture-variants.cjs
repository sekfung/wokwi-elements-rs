#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const output = path.join(root, 'src', 'generated', 'hx711_variants.json');

async function main() {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    await page.goto(`file://${path.join(root, 'tools', 'reference-page.html')}`);
    await page.waitForFunction(() => window.elements);
    const variants = await page.evaluate(async () => {
      const result = {};
      for (const type of ['50kg', '5kg', 'gauge']) {
        const element = document.createElement('wokwi-hx711');
        element.type = type;
        document.body.append(element);
        await element.updateComplete;
        result[type] = {
          html: element.shadowRoot.innerHTML,
          styles: element.constructor.styles?.cssText ?? '',
          pinInfo: element.pinInfo,
        };
        element.remove();
      }
      return result;
    });
    fs.writeFileSync(output, `${JSON.stringify(variants, null, 2)}\n`);
    console.log(`captured hx711 variants -> ${output}`);
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
