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
  ['timestamps', 'startup', 'live'].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description: `should exclude an ignored real directory root from ${scenario}`,
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), 'rspack-ignored-root-'),
      );
      const app = path.join(root, 'app');
      const target = path.join(root, 'outside');
      const nestedTarget = path.join(root, 'nested-target');
      for (const dir of [app, target, nestedTarget]) fs.mkdirSync(dir);
      const files = [
        path.join(target, 'index.js'),
        path.join(nestedTarget, 'nested.js'),
      ];
      const past = new Date(Date.now() - 60_000);
      for (const file of files) {
        fs.writeFileSync(file, 'initial');
        fs.utimesSync(file, past, past);
      }
      fs.symlinkSync(nestedTarget, path.join(target, 'nested'), 'junction');
      const link = path.join(app, 'link');
      fs.symlinkSync(target, link, 'junction');
      await sleep(2100);
      const startTime = Date.now();
      if (scenario === 'startup') fs.writeFileSync(files[0], 'before watching');
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored: (file) =>
          file === target || file.startsWith(`${target}${path.sep}`),
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changes = new Set();
      const events = new Set();
      let error;
      if (native) {
        watcher.watch(
          [[], []],
          [[link], []],
          [[], []],
          BigInt(startTime),
          (err, batch) => {
            if (err) error = err;
            else for (const file of batch.changedFiles) changes.add(file);
          },
          (event) => events.add(event.path),
        );
      } else {
        watcher.on('aggregated', (files) => {
          for (const file of files) changes.add(file);
        });
        watcher.on('change', (file) => events.add(file));
        watcher.watch({ directories: [link], startTime });
      }
      try {
        await sleep(500);
        if (scenario === 'timestamps') {
          const paths = native
            ? watcher.collectTimeInfoEntries().fileTimestamps.entries.map(
                (entry) => entry.path,
              )
            : [...watcher.getTimeInfoEntries().keys()];
          expect(
            paths.some((file) => file.startsWith(`${link}${path.sep}`)),
          ).toBe(false);
          expect(
            paths.some((file) => file.startsWith(`${target}${path.sep}`)),
          ).toBe(false);
        } else if (scenario === 'live') {
          changes.clear();
          events.clear();
          for (const file of files) fs.writeFileSync(file, 'while watching');
          await sleep(700);
        }
        expect([...changes]).toEqual([]);
        expect([...events]).toEqual([]);
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
