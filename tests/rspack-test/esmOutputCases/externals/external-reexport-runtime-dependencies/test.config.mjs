import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export default {
  snapshotFileFilter() {
    return false;
  },
  findBundle() {
    return [];
  },
  afterExecute(options) {
    for (const config of options) {
      const output = path.join(config.output.path, config.name);
      for (const name of ['ext.mjs', 'side.mjs']) {
        fs.copyFileSync(
          path.join(import.meta.dirname, name),
          path.join(output, name),
        );
      }
      const entry = pathToFileURL(path.join(output, 'main.mjs')).href;
      const result = execFileSync(
        process.execPath,
        [
          '--input-type=module',
          '-e',
          `const main = await import(${JSON.stringify(entry)});
           console.log(JSON.stringify({
             value: main.value,
             helperLoaded: globalThis.helperLoaded,
             bridgeLoaded: globalThis.bridgeLoaded === true
           }));`,
        ],
        { encoding: 'utf-8' },
      );
      expect(JSON.parse(result)).toEqual({
        value: 42,
        helperLoaded: true,
        bridgeLoaded: false,
      });
    }
  },
};
