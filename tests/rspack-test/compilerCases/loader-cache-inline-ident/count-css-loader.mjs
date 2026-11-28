import fs from 'node:fs';

export default function countCssLoader(source) {
  const { counterFile } = this.getOptions();
  fs.appendFileSync(counterFile, 'x');
  return source.replaceAll('COLOR', 'crimson');
}
