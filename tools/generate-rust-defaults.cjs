#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const sourcePath = path.join(root, 'reference/contracts/upstream-default.json');
const outDir = path.join(root, 'src/generated');
const outPath = path.join(outDir, 'defaults.json');

const captured = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));
const defaults = captured.map((component) => ({
  tag: component.tag,
  html: component.shadowHtml.replace(/<!--[^>]*-->/g, ''),
  styles: component.styles ?? '',
  pinInfo: component.pinInfo ?? null,
}));

fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(defaults, null, 2)}\n`);
console.log(`wrote ${defaults.length} default renders -> ${outPath}`);
