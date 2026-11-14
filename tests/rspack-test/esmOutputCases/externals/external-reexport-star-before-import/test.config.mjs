import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const expectedOrder = {
  plain: ['ext', 'side'],
  rslib: ['ext', 'middle', 'other', 'side'],
  wrapped: ['ext', 'other'],
  alias: ['ext', 'other', 'middle'],
  namespace: ['middle', 'ext', 'other'],
};

export default {
  findBundle() {
    return [];
  },
  afterExecute(options) {
    for (const config of options) {
      const rslib = config.name !== 'plain';
      const wrapped = config.name === 'wrapped';
      const output = path.join(config.output.path, config.name);
      for (const name of ['ext.mjs', 'middle-side.mjs', 'other.mjs', 'side.mjs']) {
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
             order: ${wrapped} ? globalThis.__order.filter(x => x !== 'side') : globalThis.__order,
             before,
             after: main.value,
             ownBefore,
             ownAfter: main.own ?? null,
             other: main.other ?? null,
             middle: main.middle ?? null,
             hasDefault: Object.hasOwn(main, 'default'),
             hasShared: Object.hasOwn(main, 'shared')
           }));`,
        ],
        { encoding: 'utf-8' },
      );
      expect(JSON.parse(result)).toEqual({
        order: expectedOrder[config.name],
        before: 1,
        after: 2,
        ownBefore: config.name === 'rslib' ? 1 : null,
        ownAfter: config.name === 'rslib' ? 2 : null,
        other: rslib ? 42 : null,
        middle: ['alias', 'namespace'].includes(config.name) ? 7 : null,
        hasDefault: false,
        hasShared: !rslib,
      });
    }
  },
};
