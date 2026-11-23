import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { NativeWatcher, Watchpack, waitFor } from '../_helpers/watcher.mjs';

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [false, true].map((nativeWatcher) => ({
  name: nativeWatcher ? 'native' : 'watchpack',
  description:
    'should reattach when a linked directory is recreated at the same path before rewatch',
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-link-recreated-'),
    );
    const source = path.join(root, 'app');
    const target = path.join(root, 'target');
    fs.mkdirSync(source);
    fs.mkdirSync(target);
    fs.writeFileSync(path.join(target, 'index.js'), 'initial');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(path.join(target, 'index.js'), past, past);
    const link = path.join(source, 'link');
    fs.symlinkSync(target, link, 'junction');
    await sleep(2100);
    const options = { aggregateTimeout: 50, followSymlinks: true };
    let watcher = nativeWatcher
      ? new NativeWatcher(options)
      : new Watchpack(options);
    const changes = new Set();
    let error;
    const callback = (err, batch) => {
      if (err) error = err;
      else for (const file of batch.changedFiles) changes.add(file);
    };
    const watch = (initial) => {
      if (nativeWatcher)
        watcher.watch(
          [[], []],
          [initial ? [link] : [], []],
          [[], []],
          BigInt(Date.now()),
          callback,
          () => {},
        );
      else {
        // NodeWatchFileSystem creates a new Watchpack for each watch cycle.
        watcher.close();
        watcher = new Watchpack(options);
        watcher.on('aggregated', (files) => {
          for (const file of files) changes.add(file);
        });
        watcher.watch({
          files: [],
          directories: [link],
          missing: [],
          startTime: Date.now(),
        });
      }
    };
    watch(true);
    try {
      await sleep(500);
      changes.clear();
      fs.renameSync(target, path.join(root, 'previous'));
      fs.mkdirSync(target);
      fs.writeFileSync(path.join(target, 'index.js'), 'replacement');
      await sleep(500);
      watch(false);
      await sleep(500);
      changes.clear();
      fs.writeFileSync(path.join(target, 'index.js'), 'after rewatch');
      await waitFor(() => changes.has(link));
      expect(error).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
