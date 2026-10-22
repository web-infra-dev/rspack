import fs from 'node:fs';
import path from 'node:path';

export function findTestFile(
  dir: string,
  name: 'test.config' | 'test.filter' | 'test',
) {
  return ['mjs', 'cjs', 'js']
    .map((extension) => path.join(dir, `${name}.${extension}`))
    .find((file) => fs.existsSync(file));
}

export function readTestFile<T>(file: string): T {
  const config = require(file);
  return file.endsWith('.mjs') ? config.default : config;
}
