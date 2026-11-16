#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const sourcePath = path.join(
  root,
  'reference',
  'wokwi-elements-1.9.2',
  'dist',
  'esm',
  'biaxial-stepper-element.js',
);
const source = fs.readFileSync(sourcePath, 'utf8');
global.OUTER_OFFSET = 9;
global.INNER_OFFSET = 4.7;
global.ORNATE_OUTER_OFFSET = 9;
global.ORNATE_INNER_OFFSET = 5;
const marker = 'this.handMap = {';
const start = source.indexOf(marker) + 'this.handMap = '.length;
let depth = 0;
let end = start;
for (; end < source.length; end += 1) {
  if (source[end] === '{') depth += 1;
  if (source[end] === '}') {
    depth -= 1;
    if (depth === 0) {
      end += 1;
      break;
    }
  }
}
const objectLiteral = source.slice(start, end);
const handMap = new Function(`return (${objectLiteral});`)();
const result = {};
for (const [name, factory] of Object.entries(handMap)) {
  const fnSource = factory.toString();
  const pathMatch = fnSource.match(/path:\s*`([\s\S]*?)`/);
  if (!pathMatch) throw new Error(`no path template for ${name}`);
  const offsets = factory(30);
  result[name] = {
    xOff: offsets.xOff,
    yOff: offsets.yOff,
    pathTemplate: pathMatch[1].replaceAll('${len}', '{len}'),
  };
}
const output = path.join(root, 'src', 'components', 'biaxial_hands.json');
fs.writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`);
console.log(`captured ${Object.keys(result).length} biaxial hand templates -> ${output}`);
