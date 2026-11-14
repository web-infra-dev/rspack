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
    'overlap',
    'overlap-reversed',
    'parent-only',
    'link-only',
    'ignored-target-file',
    'ignored-alias-file',
    'ignored-parent-directory',
    'no-ignore',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description:
      'should filter overlapping link contexts independently during startup',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-overlapping-contexts-',
        ),
      );
      const app = path.join(root, 'app');
      const target = path.join(root, 'target');
      const container = path.join(app, 'container');
      const link = path.join(
        scenario === 'ignored-parent-directory' ? container : app,
        'link',
      );
      const file = path.join(target, 'value.js');
      const alias = path.join(link, 'value.js');
      fs.mkdirSync(app);
      if (scenario === 'ignored-parent-directory') fs.mkdirSync(container);
      fs.mkdirSync(target);
      fs.writeFileSync(file, 'initial');
      const past = new Date(Date.now() - 60_000);
      fs.utimesSync(file, past, past);
      fs.symlinkSync(target, link, 'junction');
      await sleep(2100);
      const startTime = Date.now();
      await sleep(50);
      fs.writeFileSync(file, 'changed before watching');
      const directories =
        scenario === 'parent-only'
          ? [app]
          : scenario === 'link-only'
            ? [link]
            : scenario === 'overlap-reversed'
              ? [link, app]
              : [app, link];
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored(entry) {
          switch (scenario) {
            case 'no-ignore':
              return false;
            case 'ignored-parent-directory':
              return entry === container;
            case 'ignored-target-file':
              return entry === file;
            case 'ignored-alias-file':
              return entry === alias;
            default:
              return (
                entry === target || entry.startsWith(`${target}${path.sep}`)
              );
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
      try {
        if (native) {
          watcher.watch(
            [[], []],
            [directories, []],
            [[], []],
            BigInt(startTime),
            (error, batch) => {
              if (error) failure = error;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        } else {
          watcher.on('aggregated', record);
          watcher.watch({ directories, startTime });
        }
        const expected =
          scenario === 'no-ignore'
            ? [app, link]
            : scenario === 'link-only'
              ? []
              : scenario === 'ignored-alias-file' ||
                  scenario === 'ignored-parent-directory'
                ? [link]
                : [app];
        if (expected.length > 0) {
          const deadline = Date.now() + 5000;
          while (
            !expected.every((entry) => changes.has(entry)) &&
            Date.now() < deadline
          )
            await sleep(10);
        }
        await sleep(500);
        expect(failure).toBeUndefined();
        expect([...changes].sort()).toEqual(expected);
        expect([...removals]).toEqual([]);
        const timestamps = native
          ? watcher
              .collectTimeInfoEntries()
              .fileTimestamps.entries.map((entry) => entry.path)
          : [...watcher.getTimeInfoEntries().keys()];
        if (expected.includes(app)) expect(timestamps).toContain(alias);
        if (scenario === 'link-only') expect(timestamps).not.toContain(alias);
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
