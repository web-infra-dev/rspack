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
      const rootless = config.name.startsWith('rootless-');
      const nested = config.name.startsWith('nested-');
      const independentEntries = config.name.startsWith('independent-entries-');
      const output = path.join(config.output.path, config.name);
      for (const name of ['ext.mjs', 'side.mjs', 'tail.mjs']) {
        fs.copyFileSync(
          path.join(import.meta.dirname, name),
          path.join(output, name),
        );
      }
      // These entries are independent: loading the second entry must not adopt
      // the first entry's external re-export order in their shared chunk.
      const entry = pathToFileURL(
        path.join(output, independentEntries ? 'second.mjs' : 'main.mjs'),
      ).href;
      const result = execFileSync(
        process.execPath,
        [
          '--input-type=module',
          '-e',
          `globalThis.order = [];
           const main = await import(${JSON.stringify(entry)});
           const before = [main.value, main.default ?? null, main.quoted ?? null, main.direct ?? null];
           main.update();
           console.log(JSON.stringify({
             before,
             after: [main.value, main.default ?? null, main.quoted ?? null, main.direct ?? null],
             order: globalThis.order
           }));`,
        ],
        { encoding: 'utf-8' },
      );
      const { before, after, order } = JSON.parse(result);
      expect(before).toEqual([
        42,
        rootless ? 42 : null,
        rootless ? 42 : null,
        nested ? 42 : null,
      ]);
      expect(after).toEqual([
        43,
        rootless ? 43 : null,
        rootless ? 43 : null,
        nested ? 43 : null,
      ]);
      const prerequisite = rootless || nested ? 'setup' : 'side';
      expect(order.indexOf(prerequisite)).toBeGreaterThanOrEqual(0);
      expect(order.indexOf('ext')).toBeGreaterThan(order.indexOf(prerequisite));
    }
  },
};
