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
  [
    'target-file',
    'target-tree',
    'target-directory',
    'alias',
    'no-ignore',
    'startup-target-file',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description:
      'should apply ignored to the target of a file link discovered in a context',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-ignored-context-file-links-',
        ),
      );
      const app = path.join(root, 'app');
      const outside = path.join(root, 'outside');
      const target = path.join(outside, 'value.js');
      const link = path.join(app, 'link.js');
      const witness = path.join(app, 'witness.js');
      const startup = scenario === 'startup-target-file';
      fs.mkdirSync(app);
      fs.mkdirSync(outside);
      fs.writeFileSync(target, 'before');
      fs.writeFileSync(witness, 'before');
      const old = new Date(Date.now() - 60_000);
      fs.utimesSync(target, old, old);
      fs.utimesSync(witness, old, old);
      fs.symlinkSync(target, link, 'file');
      await sleep(2200);
      const startTime = Date.now();
      if (startup) {
        await sleep(50);
        fs.writeFileSync(target, 'changed during build');
        await sleep(500);
      }
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored(entry) {
          switch (scenario) {
            case 'target-file':
            case 'startup-target-file':
              return entry === target;
            case 'target-tree':
              return (
                entry === outside || entry.startsWith(`${outside}${path.sep}`)
              );
            case 'target-directory':
              return entry === outside;
            case 'alias':
              return entry === link;
            default:
              return false;
          }
        },
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changes = new Set();
      const removals = new Set();
      let failure;
      const record = (changed, removed) => {
        for (const entry of changed) changes.add(entry);
        for (const entry of removed) removals.add(entry);
      };
      const waitFor = async (predicate) => {
        const deadline = Date.now() + 5000;
        while (!predicate() && Date.now() < deadline) await sleep(10);
        expect(predicate()).toBe(true);
      };
      try {
        if (native)
          watcher.watch(
            [[], []],
            [[app], []],
            [[], []],
            BigInt(startTime),
            (error, batch) => {
              if (error) failure = error;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        else {
          watcher.on('aggregated', record);
          watcher.watch({ directories: [app], startTime });
        }
        await waitFor(() => {
          const entries = native
            ? watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.map((entry) => entry.path)
            : [...watcher.getTimeInfoEntries().keys()];
          return entries.includes(witness);
        });
        await sleep(500);
        if (!startup) {
          changes.clear();
          removals.clear();
          fs.writeFileSync(target, 'live edit');
        }
        const expected =
          scenario === 'target-directory' || scenario === 'no-ignore'
            ? [app]
            : [];
        if (expected.length) await waitFor(() => changes.has(app));
        await sleep(500);
        expect([...changes]).toEqual(expected);
        expect([...removals]).toEqual([]);
        if (scenario === 'target-file' || scenario === 'target-tree') {
          fs.unlinkSync(link);
          fs.symlinkSync(target, link, 'file');
          await waitFor(() => changes.has(app));
        }
        changes.clear();
        fs.writeFileSync(witness, 'ordinary edit');
        await waitFor(() => changes.has(app));
        expect(failure).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
