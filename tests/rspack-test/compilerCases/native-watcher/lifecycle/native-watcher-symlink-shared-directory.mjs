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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should keep a file target watched after removing an overlapping directory link',
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-shared-link-'),
    );
    const app = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    fs.mkdirSync(app);
    fs.mkdirSync(path.join(outside, 'sub'), { recursive: true });
    const target = path.join(outside, 'sub', 'index.js');
    fs.writeFileSync(target, 'before');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(target, past, past);
    const directory = path.join(app, 'directory');
    const file = path.join(app, 'file.js');
    fs.symlinkSync(outside, directory, 'junction');
    fs.symlinkSync(target, file, 'file');
    await sleep(2100);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const native = new NativeWatcher(options);
    const watchpack = new Watchpack(options);
    const changes = [new Set(), new Set()];
    let failure;
    const callback = (error, batch) => {
      if (error) failure = error;
      else for (const path of batch.changedFiles) changes[0].add(path);
    };
    watchpack.on('aggregated', (files) => {
      for (const path of files) changes[1].add(path);
    });
    native.watch(
      [[file], []],
      [[directory], []],
      [[], []],
      BigInt(Date.now()),
      callback,
      () => {},
    );
    watchpack.watch({
      files: [file],
      directories: [directory],
      startTime: Date.now(),
    });
    try {
      await sleep(500);
      native.watch(
        [[], []],
        [[], [directory]],
        [[], []],
        BigInt(Date.now()),
        callback,
        () => {},
      );
      watchpack.watch({
        files: [file],
        directories: [],
        startTime: Date.now(),
      });
      await sleep(500);
      for (const set of changes) set.clear();
      fs.writeFileSync(target, 'after removing directory');
      const deadline = Date.now() + 5000;
      while (!changes.every((set) => set.has(file)) && Date.now() < deadline)
        await sleep(10);
      expect(failure).toBeUndefined();
      expect([...changes[1]]).toEqual([file]);
      expect([...changes[0]]).toEqual([file]);
    } finally {
      watchpack.close();
      await native.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
};
