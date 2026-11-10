import fs from 'node:fs';
import path from 'node:path';

export default {
  findBundle() {
    return [];
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
      'export { yv } from "./y.js";',
      'import "./x.js";',
    ]);
  },
};
