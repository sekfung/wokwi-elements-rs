#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const sourcePath = path.join(root, 'reference', 'contracts', 'upstream-default.json');
const outPath = path.join(root, 'reference', 'contracts', 'components.json');

function compactDefault(value) {
  if (Array.isArray(value)) {
    if (value.length > 64) {
      return { kind: 'zeros', length: value.length };
    }
    return { kind: 'value', value };
  }
  if (value && typeof value === 'object' && value.data) {
    return { kind: 'zeros', length: value.data.length ?? 0 };
  }
  return { kind: 'value', value };
}

const captured = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));
const components = captured.map((component) => ({
  tag: component.tag,
  className: component.className,
  properties: Object.fromEntries(
    Object.entries(component.properties).map(([name, property]) => [
      name,
      {
        type: property.type,
        attribute: property.attribute,
        default: compactDefault(property.default),
      },
    ]),
  ),
}));

fs.writeFileSync(outPath, `${JSON.stringify(components, null, 2)}\n`);
console.log(`wrote ${components.length} component contracts -> ${outPath}`);
