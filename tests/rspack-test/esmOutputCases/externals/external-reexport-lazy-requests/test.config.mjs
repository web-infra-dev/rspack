import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export default {
  snapshotFileFilter(file) {
    return file === 'child.mjs';
  },
  findBundle() {
    return [];
  },
  afterExecute({ output }) {
    for (const name of ['first.mjs', 'second.mjs']) {
      fs.copyFileSync(
        path.join(import.meta.dirname, name),
        path.join(output.path, name),
      );
    }
    const orders = {};
    for (const name of ['lazy', 'asset', 'async']) {
      const entry = pathToFileURL(path.join(output.path, `${name}.mjs`)).href;
      // Each entry needs a fresh ESM module cache to observe its first request.
      orders[name] = JSON.parse(
        execFileSync(
          process.execPath,
          [
            '--input-type=module',
            '-e',
            `const order = globalThis.order = [];
             const main = await import(${JSON.stringify(entry)});
             const startup = [...order];
             const loaded = ${name === 'async' ? 'await main.load()' : 'main'};
             console.log(JSON.stringify([startup, order, loaded.value]));`,
          ],
          { encoding: 'utf-8' },
        ),
      );
    }
    const expected = ['first', 'second'];
    expect(orders).toEqual({
      lazy: [expected, expected, 42],
      asset: [expected, expected, 42],
      async: [[], expected, 42],
    });
  },
};
