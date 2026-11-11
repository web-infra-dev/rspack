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

let tracing = false;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
  [false, true].map((missing) => ({
    name: `${native ? 'native' : 'watchpack'}-${missing ? 'missing' : 'file'}`,
    description:
      'should report a dangling link replacement and follow its new target',
    async run() {
      if (native && process.env.RSPACK_WATCHER_TRACE && !tracing) {
        await require('@rspack/core').experiments.globalTrace.register(
          'rspack_watcher::fs_event=debug',
          'logger',
          'stdout',
        );
        tracing = true;
      }
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
      const changes = new Set();
      const removals = new Set();
      let error;
      const record = (changed, removed) => {
        if (process.env.RSPACK_WATCHER_TRACE)
          console.log('dangling retarget callback', {
            native,
            missing,
            changed: [...changed],
            removed: [...removed],
          });
        for (const file of changed) changes.add(file);
        for (const file of removed) removals.add(file);
      };
      if (!native) watcher.on('aggregated', record);
      const watch = (initial) => {
        if (native)
          watcher.watch(
            [initial && !missing ? [link] : [], []],
            [[], []],
            [initial && missing ? [link] : [], []],
            BigInt(Date.now()),
            (err, batch) => {
              if (err) error = err;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        else
          watcher.watch({
            files: missing ? [] : [link],
            missing: missing ? [link] : [],
            startTime: Date.now(),
          });
      };
      const waitFor = async (predicate) => {
        const deadline = Date.now() + 5000;
        while (!predicate() && Date.now() < deadline) await sleep(10);
        expect(predicate()).toBe(true);
      };
      try {
        watch(true);
        await sleep(500);
        changes.clear();
        removals.clear();
        const replacement = path.join(app, 'replacement.js');
        fs.symlinkSync(targets[1], replacement, 'file');
        if (process.env.RSPACK_WATCHER_TRACE)
          console.log('dangling retarget replacement', {
            native,
            missing,
            link,
            previousMtime: fs.lstatSync(link).mtimeMs,
            nextMtime: fs.lstatSync(replacement).mtimeMs,
          });
        fs.renameSync(replacement, link);
        await waitFor(() => changes.has(link));
        expect([...removals]).toEqual([]);
        watch(false);
        await sleep(500);
        changes.clear();
        fs.writeFileSync(targets[1], 'new target appears');
        await waitFor(() => changes.has(link));
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
