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
      const commonjs = config.name.startsWith('commonjs-');
      const attributes = config.name.startsWith('attributes-');
      const output = path.join(config.output.path, config.name);
      for (const name of [
        'first.mjs',
        'first with space.mjs',
        '数据 with space.json',
        'attribute-ext.mjs',
        'ext.mjs',
      ]) {
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
          `globalThis.order = [];
           const main = await import(${JSON.stringify(entry)});
           const before = main.value;
           main.update();
           console.log(JSON.stringify({
             before,
             after: main.value,
             observed: main.observed ?? null,
             order: globalThis.order
           }));`,
        ],
        { encoding: 'utf-8' },
      );
      const { order, ...bindings } = JSON.parse(result);
      expect(bindings).toEqual({
        before: 42,
        after: 43,
        observed: commonjs ? null : 1,
      });
      expect(order.filter((item) => item !== 'helper')).toEqual(
        attributes ? [] : ['first', 'ext'],
      );
      expect(order.filter((item) => item === 'helper')).toEqual(
        commonjs ? ['helper'] : [],
      );
    }
  },
};
