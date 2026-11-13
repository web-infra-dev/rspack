import fs from 'node:fs';

export default function cssLoader(source) {
  const { color, counterFile } = this.getOptions();
  fs.appendFileSync(counterFile, 'x');
  return source.replace('COLOR', color);
}
