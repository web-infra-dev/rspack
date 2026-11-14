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
export default [false, true].map((native) => ({
  name: native ? 'native' : 'watchpack',
  description:
    'should recover a context file link target restored with its old mtime',
  async run() {
    const root = fs.mkdtempSync(
      path.join(
        fs.realpathSync.native(os.tmpdir()),
        'rspack-file-link-recovery-',
      ),
    );
    const app = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    fs.mkdirSync(app);
    fs.mkdirSync(outside);
    const target = path.join(outside, 'value.js');
    fs.writeFileSync(target, 'initial');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(target, past, past);
    const link = path.join(app, 'link.js');
    fs.symlinkSync(target, link, 'file');
    await sleep(2100);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const watcher = native
      ? new NativeWatcher(options)
      : new Watchpack(options);
    const changes = new Set();
    let error;
    try {
      if (native)
        watcher.watch(
          [[], []],
          [[app], []],
          [[], []],
          BigInt(Date.now()),
          (err, batch) => {
            if (err) error = err;
            else for (const file of batch.changedFiles) changes.add(file);
          },
          () => {},
        );
      else {
        watcher.on('aggregated', (files) => {
          for (const file of files) changes.add(file);
        });
        watcher.watch({ directories: [app], startTime: Date.now() });
      }
      await sleep(500);
      changes.clear();
      fs.unlinkSync(target);
      await sleep(1000);
      expect([...changes]).toEqual([]);
      const replacement = path.join(outside, 'replacement.js');
      fs.writeFileSync(replacement, 'changed contents');
      fs.utimesSync(replacement, past, past);
      fs.renameSync(replacement, target);
      const deadline = Date.now() + 5000;
      while (!changes.has(app) && Date.now() < deadline) await sleep(10);
      expect([...changes]).toEqual([app]);
      expect(error).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
