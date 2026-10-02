import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const source = new URL('../../../.local/screens/workbench.svg', import.meta.url);
const output = new URL('./reference-workspace-gold.svg', import.meta.url);
const orange = '#ed660c';
const gold = '#e2a35f';

const original = readFileSync(source, 'utf8');
const occurrences = original.split(orange).length - 1;
if (occurrences === 0 && !original.includes(gold)) {
  throw new Error(`Expected ${orange} or ${gold} in the rendered workbench`);
}

writeFileSync(output, original.replaceAll(orange, gold));
console.log(occurrences ? `Recolored ${occurrences} accent cells to ${gold}` : `Accent already uses ${gold}`);
