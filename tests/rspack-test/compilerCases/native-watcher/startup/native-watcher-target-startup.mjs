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
  [
    'missing-target',
    'retained-retarget',
    'no-follow',
    'ignored-target',
    'missing-registration',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description: 'should backfill a newly subscribed file target',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-target-startup-',
        ),
      );
      const app = path.join(root, 'app');
      fs.mkdirSync(app);
      const targets = ['a', 'b'].map((name) => {
        const directory = path.join(root, name);
        fs.mkdirSync(directory);
        const file = path.join(directory, 'value.js');
        fs.writeFileSync(file, name);
        const past = new Date(Date.now() - 60_000);
        fs.utimesSync(file, past, past);
        return file;
      });
      const link = path.join(app, 'link.js');
      fs.symlinkSync(targets[0], link, 'file');
      await sleep(2100);
      const missing = scenario === 'missing-registration';
      const options = {
        followSymlinks: scenario !== 'no-follow',
        aggregateTimeout: 50,
        ...(scenario === 'ignored-target'
          ? { ignored: (file) => file === targets[0] }
          : {}),
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const collected = createEventCollector();
      const { changes, removals, record, callback } = collected;
      if (!native) watcher.on('aggregated', record);
      const watch = (initial, startTime) => {
        if (native)
          watcher.watch(
            [initial && !missing ? [link] : [], []],
            [[], []],
            [initial && missing ? [link] : [], []],
            BigInt(startTime),
            callback,
            () => {},
          );
        else
          watcher.watch({
            files: missing ? [] : [link],
            missing: missing ? [link] : [],
            startTime,
          });
      };
      try {
        if (scenario !== 'retained-retarget') {
          const startTime = Date.now();
          fs.unlinkSync(targets[0]);
          watch(true, startTime);
          if (scenario === 'missing-target')
            await waitFor(() => removals.has(link));
          else {
            await sleep(700);
            expect([...removals]).toEqual([]);
          }
          expect([...changes]).toEqual([]);
        } else {
          watch(true, Date.now());
          await sleep(500);
          fs.unlinkSync(link);
          fs.symlinkSync(targets[1], link, 'file');
          await waitFor(() => changes.has(link));
          await sleep(300);
          changes.clear();
          removals.clear();
          const startTime = Date.now();
          await sleep(25);
          fs.writeFileSync(targets[1], 'edited while the build was running');
          await sleep(300);
          expect([...changes]).toEqual([]);
          watch(false, startTime);
          await waitFor(() => changes.has(link));
          expect([...removals]).toEqual([]);
        }
        expect(collected.error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
