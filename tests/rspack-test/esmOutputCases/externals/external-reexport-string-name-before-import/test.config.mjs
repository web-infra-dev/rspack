import fs from 'node:fs';
import path from 'node:path';

const sourceDir = import.meta.dirname;

export default {
  beforeExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    for (const name of ['ext.mjs', 'side.mjs']) {
      fs.copyFileSync(
        path.join(sourceDir, name),
        path.join(config.output.path, name),
      );
    }
  },
  findBundle() {
    return ['main.mjs'];
  },
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, 'main.mjs'),
      'utf-8',
    );
    const ordered = source
      .split('\n')
      .map((line) => line.trim())
      .filter(
        (line) => line.startsWith('import ') || line.startsWith('export '),
      );
    const extIndex = ordered.findIndex((line) => line.includes('./ext.mjs'));
    const sideIndex = ordered.findIndex((line) => line.includes('./side.mjs'));
    expect(extIndex).toBeGreaterThanOrEqual(0);
    expect(sideIndex).toBeGreaterThan(extIndex);
    expect(source).toMatch(/["']x-y["']/);
    expect(source).toMatch(/export \{[^}]*value/);
  },
};
