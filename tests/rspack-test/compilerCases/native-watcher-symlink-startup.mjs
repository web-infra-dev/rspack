import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default ['directory-edit', 'directory-remove', 'file-edit'].map(
  (scenario) => ({
    name: scenario,
    description:
      'should catch a linked target edit between its initial scan and watch subscription',
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), 'rspack-link-startup-'),
      );
      const context = path.join(root, 'app');
      const outside = path.join(root, 'outside');
      fs.mkdirSync(context);
      fs.mkdirSync(outside);
      const link = path.join(context, 'link');
      if (scenario !== 'file-edit') fs.symlinkSync(outside, link, 'junction');
      const past = new Date(Date.now() - 60_000);
      for (const name of ['a.js', 'b.js']) {
        const file = path.join(outside, name);
        fs.writeFileSync(file, 'before');
        fs.utimesSync(file, past, past);
        if (scenario === 'file-edit')
          fs.symlinkSync(file, path.join(context, name), 'file');
      }
      await sleep(2100);
      let first;
      let changed = false;
      const changes = new Set();
      let failure;
      const watcher = new NativeWatcher({
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored(entry) {
          if (
            path.dirname(entry) ===
              (scenario === 'file-edit' ? context : link) &&
            entry.endsWith('.js')
          ) {
            if (!first) first = entry;
            else if (entry !== first && !changed) {
              // The first entry has been scanned; the external directory is not
              // subscribed yet. Do not depend on directory enumeration order.
              changed = true;
              if (scenario === 'directory-remove') fs.unlinkSync(first);
              else fs.writeFileSync(first, 'during scan');
            }
          }
          return false;
        },
      });
      watcher.watch(
        [[], []],
        [[context], []],
        [[], []],
        BigInt(Date.now()),
        (error, batch) => {
          if (error) failure = error;
          else for (const file of batch.changedFiles) changes.add(file);
        },
        () => {},
      );
      try {
        const deadline = Date.now() + 5000;
        while (!changes.has(context) && Date.now() < deadline) await sleep(10);
        expect(failure).toBeUndefined();
        expect(changed).toBe(true);
        expect(changes.has(context)).toBe(true);
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  }),
);
