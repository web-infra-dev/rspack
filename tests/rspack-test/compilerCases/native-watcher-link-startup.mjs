import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);
const Watchpack = require(
  path.join(
    path.dirname(require.resolve('@rspack/core/package.json')),
    'compiled/watchpack',
  ),
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
  ['file', 'intermediate-directory'].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description: 'should backfill link replacements before the initial watch',
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync.native(os.tmpdir()), 'rspack-link-startup-'),
      );
      const oldTarget = path.join(root, 'old');
      const newTarget = path.join(root, 'new');
      const past = new Date(Date.now() - 60_000);
      for (const target of [oldTarget, newTarget]) {
        fs.mkdirSync(target);
        const file = path.join(target, 'value.js');
        fs.writeFileSync(file, target);
        fs.utimesSync(file, past, past);
        fs.utimesSync(target, past, past);
      }
      const directory = scenario === 'intermediate-directory';
      const link = path.join(root, 'link');
      const targetPath = (target) =>
        directory ? target : path.join(target, 'value.js');
      fs.symlinkSync(
        targetPath(oldTarget),
        link,
        directory ? 'junction' : 'file',
      );
      const registered = directory ? path.join(root, 'entry.js') : link;
      if (directory)
        fs.symlinkSync(path.join(link, 'value.js'), registered, 'file');
      await sleep(2100);
      const startTime = Date.now();
      await sleep(25);
      fs.unlinkSync(link);
      fs.symlinkSync(
        targetPath(newTarget),
        link,
        directory ? 'junction' : 'file',
      );
      const options = { followSymlinks: true, aggregateTimeout: 50 };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changed = new Set();
      const removed = new Set();
      let error;
      const record = (changes, removals) => {
        for (const file of changes) changed.add(file);
        for (const file of removals) removed.add(file);
      };
      try {
        if (native)
          watcher.watch(
            [[registered], []],
            [[], []],
            [[], []],
            BigInt(startTime),
            (err, batch) => {
              if (err) error = err;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        else {
          watcher.on('aggregated', record);
          watcher.watch({ files: [registered], startTime });
        }
        const deadline = Date.now() + 5000;
        while (!changed.has(registered) && Date.now() < deadline)
          await sleep(10);
        expect([...changed]).toEqual([registered]);
        expect([...removed]).toEqual([]);
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
