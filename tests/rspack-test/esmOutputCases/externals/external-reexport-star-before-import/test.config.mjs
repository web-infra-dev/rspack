import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export default {
  findBundle() {
    return [];
  },
  afterExecute(options) {
    for (const config of options) {
      const rslib = config.name === 'rslib';
      const output = path.join(config.output.path, config.name);
      for (const name of ['ext.mjs', 'other.mjs', 'side.mjs']) {
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
          `globalThis.__order = [];
           const main = await import(${JSON.stringify(entry)});
           const before = main.value;
           const ownBefore = main.own ?? null;
           main.update();
           main.bump?.();
           console.log(JSON.stringify({
             order: globalThis.__order,
             before,
             after: main.value,
             ownBefore,
             ownAfter: main.own ?? null,
             other: main.other ?? null,
             hasDefault: Object.hasOwn(main, 'default'),
             hasShared: Object.hasOwn(main, 'shared')
           }));`,
        ],
        { encoding: 'utf-8' },
      );
      expect(JSON.parse(result)).toEqual({
        order: rslib ? ['ext', 'other', 'side'] : ['ext', 'side'],
        before: 1,
        after: 2,
        ownBefore: rslib ? 1 : null,
        ownAfter: rslib ? 2 : null,
        other: rslib ? 42 : null,
        hasDefault: false,
        hasShared: !rslib,
      });
    }
  },
};
