import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = path.dirname(fileURLToPath(import.meta.url));

export function generateCss({ rules, bytes, probeColor = 'rgb(255, 0, 0)' }) {
  const chunks = [`.probe { color: ${probeColor}; }\n`];
  let size = chunks[0].length;
  let i = 0;
  const enough = () => (bytes ? size >= bytes : i >= rules);
  while (!enough()) {
    const rule = `.rule-${i} { color: rgb(${i % 255}, ${(i * 7) % 255}, ${(i * 13) % 255}); padding-top: ${i % 50}px; }\n`;
    chunks.push(rule);
    size += rule.length;
    i += 1;
  }
  const css = chunks.join('');
  fs.writeFileSync(path.join(dir, 'src/generated.css'), css);
  return { ruleCount: i, byteSize: css.length };
}

export function setProbeColor(probeColor) {
  const file = path.join(dir, 'src/generated.css');
  const content = fs.readFileSync(file, 'utf-8');
  fs.writeFileSync(
    file,
    content.replace(
      /^\.probe \{ color: [^;]+; \}/,
      `.probe { color: ${probeColor}; }`,
    ),
  );
}

export function setMarker(value) {
  fs.writeFileSync(
    path.join(dir, 'src/marker.js'),
    `export const marker = '${value}';\n`,
  );
}
