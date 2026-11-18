import fs from 'node:fs';
import path from 'node:path';

export default function (source) {
  const dependency = path.join(this.context, source.trim());
  this.addDependency(dependency);
  const color = fs.readFileSync(dependency, 'utf8').trim();
  const css = `.test { color: ${color}; }`;
  return `module.exports = ${JSON.stringify([[this.resourcePath, css, '']])};`;
}
