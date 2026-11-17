import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import {
  NativeWatcher,
  Watchpack,
  waitFor,
  createEventCollector,
} from '../_helpers/watcher.mjs';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
  ['startup-file', 'live-context'].flatMap((phase) =>
    ['overlap', 'parent-only', 'explicit-only', 'no-ignore'].map(
      (scenario) => ({
        name: `${native ? 'native' : 'watchpack'}-${phase}-${scenario}`,
        description:
          'should keep parent context notifications independent of ignored overlapping subscriptions',
        async run() {
          const root = fs.mkdtempSync(
            path.join(
              fs.realpathSync.native(os.tmpdir()),
              'rspack-independent-subscriptions-',
            ),
          );
          const app = path.join(root, 'app');
          const target = path.join(root, 'target');
          const link = path.join(app, 'link');
          const real = path.join(target, 'value.js');
          const alias = path.join(link, 'value.js');
          const witness = path.join(app, 'witness.js');
          const startup = phase === 'startup-file';
          fs.mkdirSync(app);
          fs.mkdirSync(target);
          fs.writeFileSync(real, 'before');
          fs.writeFileSync(witness, 'before');
          const old = new Date(Date.now() - 60_000);
          fs.utimesSync(real, old, old);
          fs.utimesSync(witness, old, old);
          fs.symlinkSync(target, link, 'junction');
          await sleep(2200);
          const startTime = Date.now();
          if (startup) {
            await sleep(50);
            fs.writeFileSync(real, 'changed during build');
            await sleep(500);
          }
          const files = startup && scenario !== 'parent-only' ? [alias] : [];
          const directories = scenario === 'explicit-only' ? [] : [app];
          if (!startup && scenario !== 'parent-only') directories.push(link);
          const options = {
            followSymlinks: true,
            aggregateTimeout: 50,
            ignored: (entry) =>
              scenario !== 'no-ignore' &&
              (startup
                ? entry === target || entry.startsWith(`${target}${path.sep}`)
                : entry === real),
          };
          const watcher = native
            ? new NativeWatcher(options)
            : new Watchpack(options);
          const collected = createEventCollector();
          const { changes, removals, record, callback } = collected;
          const events = new Set();
          try {
            if (native)
              watcher.watch(
                [files, []],
                [directories, []],
                [[], []],
                BigInt(startTime),
                callback,
                (event) => events.add(event.path),
              );
            else {
              watcher.on('aggregated', record);
              watcher.on('change', (file) => events.add(file));
              watcher.watch({ files, directories, startTime });
            }
            if (!startup) {
              if (directories.includes(app))
                await waitFor(() => {
                  const entries = native
                    ? watcher
                        .collectTimeInfoEntries()
                        .fileTimestamps.entries.map((e) => e.path)
                    : [...watcher.getTimeInfoEntries().keys()];
                  return entries.includes(witness) && entries.includes(alias);
                });
              await sleep(500);
              changes.clear();
              removals.clear();
              events.clear();
              fs.writeFileSync(real, 'live edit');
            }
            const expected =
              scenario === 'explicit-only'
                ? []
                : scenario === 'no-ignore'
                  ? [app, startup ? alias : link]
                  : [app];
            if (expected.length)
              await waitFor(() =>
                expected.every((entry) => changes.has(entry)),
              );
            await sleep(500);
            expect([...changes].sort()).toEqual(expected.sort());
            expect([...removals]).toEqual([]);
            expect(collected.error).toBeUndefined();
            if (scenario === 'overlap' || scenario === 'parent-only')
              expect(events.has(alias)).toBe(true);
            if (!startup && directories.includes(app)) {
              changes.clear();
              fs.writeFileSync(witness, 'ordinary edit');
              await waitFor(() => changes.has(app));
            }
          } finally {
            await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      }),
    ),
  ),
);
