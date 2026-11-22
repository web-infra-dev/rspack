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
  [false, true].map((missing) => ({
    name: `${native ? 'native' : 'watchpack'}-${missing ? 'missing' : 'file'}`,
    description:
      'should report a dangling link replacement and follow its new target',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-dangling-retarget-',
        ),
      );
      const targets = ['old', 'new'].map((name) => {
        const dir = path.join(root, name);
        fs.mkdirSync(dir);
        return path.join(dir, 'value.js');
      });
      const app = path.join(root, 'app');
      fs.mkdirSync(app);
      const link = path.join(app, 'link.js');
      fs.symlinkSync(targets[0], link, 'file');
      await sleep(2100);
      const options = { followSymlinks: true, aggregateTimeout: 50 };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const collected = createEventCollector();
      const { changes, removals, record, callback } = collected;
      if (!native) watcher.on('aggregated', record);
      const watch = (initial) => {
        if (native)
          watcher.watch(
            [initial && !missing ? [link] : [], []],
            [[], []],
            [initial && missing ? [link] : [], []],
            BigInt(Date.now()),
            callback,
            () => {},
          );
        else
          watcher.watch({
            files: missing ? [] : [link],
            missing: missing ? [link] : [],
            startTime: Date.now(),
          });
      };
      try {
        watch(true);
        await sleep(500);
        changes.clear();
        removals.clear();
        const replacement = path.join(app, 'replacement.js');
        fs.symlinkSync(targets[1], replacement, 'file');
        fs.renameSync(replacement, link);
        await waitFor(() => changes.has(link));
        expect([...removals]).toEqual([]);
        watch(false);
        await sleep(500);
        changes.clear();
        fs.writeFileSync(targets[1], 'new target appears');
        await waitFor(() => changes.has(link));
        expect(collected.error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
