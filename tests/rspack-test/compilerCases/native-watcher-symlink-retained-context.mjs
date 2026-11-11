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

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].map((removeFile) => ({
  name: removeFile ? 'remove-child-subscription' : 'retain-child-subscription',
  description:
    'should retain recursive directory coverage after rewatching overlapping links',
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-retained-context-'),
    );
    const app = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    fs.mkdirSync(app);
    fs.mkdirSync(path.join(outside, 'sub'), { recursive: true });
    const target = path.join(outside, 'sub', 'index.js');
    fs.writeFileSync(target, 'initial');
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
      else for (const changed of batch.changedFiles) changes[0].add(changed);
    };
    watchpack.on('aggregated', (files) => {
      for (const changed of files) changes[1].add(changed);
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
      for (const set of changes) set.clear();
      fs.writeFileSync(target, 'before rewatch');
      await waitFor(() => changes.every((set) => set.has(directory) && set.has(file)));

      native.watch(
        [[], removeFile ? [file] : []],
        [[], []],
        [[], []],
        BigInt(Date.now()),
        callback,
        () => {},
      );
      watchpack.watch({
        files: removeFile ? [] : [file],
        directories: [directory],
        startTime: Date.now(),
      });
      await sleep(500);
      for (const set of changes) set.clear();
      fs.writeFileSync(target, 'after rewatch');
      await waitFor(() => changes[1].has(directory));
      await waitFor(() => changes[0].has(directory));
      expect(failure).toBeUndefined();
      expect(changes[0].has(file)).toBe(!removeFile);
      expect(changes[1].has(file)).toBe(!removeFile);
    } finally {
      watchpack.close();
      await native.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
