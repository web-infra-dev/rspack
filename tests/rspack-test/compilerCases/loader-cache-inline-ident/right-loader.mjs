import fs from 'node:fs';

export default function rightLoader() {
  const { value, counterFile } = this.getOptions();
  fs.appendFileSync(counterFile, 'x');
  return `export default ${JSON.stringify(value)};`;
}
