#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const outDir = path.join(root, 'reference', 'contracts');

function discoverTags() {
  const modulesDir = path.join(
    root,
    'reference',
    'wokwi-elements-1.9.2',
    'dist',
    'esm',
  );
  const tags = new Set();
  for (const file of fs.readdirSync(modulesDir)) {
    if (!file.endsWith('-element.js') || file.endsWith('.spec.js')) {
      continue;
    }
    const source = fs.readFileSync(path.join(modulesDir, file), 'utf8');
    for (const match of source.matchAll(/customElement\('([^']+)'\)/g)) {
      tags.add(match[1]);
    }
  }
  return [...tags].sort();
}

async function main() {
  fs.mkdirSync(outDir, { recursive: true });
  const tags = discoverTags();
  const browser = await chromium.launch({ headless: true });

  try {
    const page = await browser.newPage({ viewport: { width: 1600, height: 1200 } });
    await page.goto(`file://${path.join(__dirname, 'reference-page.html')}`);
    await page.waitForFunction(() => window.elements);
    const contracts = await page.evaluate(async (componentTags) => {
      const catalog = document.querySelector('#catalog');
      const result = [];

      for (const tag of componentTags) {
        const element = document.createElement(tag);
        catalog.append(element);
        await element.updateComplete;

        const properties = {};
        for (const [name, options] of element.constructor.elementProperties ?? []) {
          const value = element[name];
          properties[name] = {
            type: options.type?.name ?? null,
            attribute:
              options.attribute === false
                ? null
                : typeof options.attribute === 'string'
                  ? options.attribute
                  : name.toLowerCase(),
            default: value === undefined ? null : value,
          };
        }

        result.push({
          tag,
          className: element.constructor.name,
          properties,
          pinInfo: element.pinInfo ?? null,
          shadowHtml: element.shadowRoot?.innerHTML ?? '',
          styles:
            typeof element.constructor.styles?.cssText === 'string'
              ? element.constructor.styles.cssText
              : null,
        });
        element.remove();
      }

      return result;
    }, tags);

    fs.writeFileSync(
      path.join(outDir, 'upstream-default.json'),
      `${JSON.stringify(contracts, null, 2)}\n`,
    );
    fs.writeFileSync(
      path.join(outDir, 'tags.json'),
      `${JSON.stringify(tags, null, 2)}\n`,
    );
    console.log(`captured ${contracts.length} components -> ${outDir}`);
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
