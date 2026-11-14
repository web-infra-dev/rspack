import fs from 'node:fs';
import path from 'node:path';

const sourceDir = import.meta.dirname;

export default {
  beforeExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    for (const name of ['x.js', 'y.js']) {
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
    expect(ordered).toEqual([
      'import { yv } from "./y.js";',
      'import "./x.js";',
      'export { yv };',
    ]);
  },
};
