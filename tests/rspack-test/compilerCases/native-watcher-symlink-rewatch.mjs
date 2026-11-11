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

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default ['file', 'directory'].map((kind) => ({
  name: kind,
  description: `should replace and release symbolic ${kind} link subscriptions on rewatch`,
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-link-rewatch-'),
    );
    const source = path.join(root, 'app');
    const targets = [path.join(root, 'a'), path.join(root, 'b')];
    fs.mkdirSync(source);
    for (const target of targets) {
      fs.mkdirSync(target);
      fs.writeFileSync(path.join(target, 'index.js'), 'initial');
      const past = new Date(Date.now() - 60_000);
      fs.utimesSync(path.join(target, 'index.js'), past, past);
      fs.writeFileSync(
        path.join(target, `${path.basename(target)}.js`),
        'distinct child',
      );
      fs.utimesSync(
        path.join(target, `${path.basename(target)}.js`),
        past,
        past,
      );
    }
    const link = path.join(source, 'link');
    const linkTarget = (target) =>
      kind === 'file' ? path.join(target, 'index.js') : target;
    fs.symlinkSync(
      linkTarget(targets[0]),
      link,
      kind === 'file' ? 'file' : 'junction',
    );
    await sleep(2100);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const native = new NativeWatcher(options);
    const watchpack = new Watchpack(options);
    const changes = [new Set(), new Set()];
    let error;
    const callback = (err, batch) => {
      if (err) error = err;
      else for (const file of batch.changedFiles) changes[0].add(file);
    };
    watchpack.on('aggregated', (files) => {
      for (const file of files) changes[1].add(file);
    });
    const files = kind === 'file' ? [link] : [];
    const directories = kind === 'directory' ? [link] : [];
    const watch = (initial, remove = false) => {
      native.watch(
        [initial ? files : [], remove ? files : []],
        [initial ? directories : [], remove ? directories : []],
        [[], []],
        BigInt(Date.now()),
        callback,
        () => {},
      );
      watchpack.watch({
        files: remove ? [] : files,
        directories: remove ? [] : directories,
        missing: [],
        startTime: Date.now(),
      });
    };
    watch(true);
    try {
      await sleep(500);
      for (const set of changes) set.clear();
      const replacement = path.join(source, 'replacement');
      fs.symlinkSync(
        linkTarget(targets[1]),
        replacement,
        kind === 'file' ? 'file' : 'junction',
      );
      fs.renameSync(replacement, link);
      await waitFor(() => changes.every((set) => set.has(link)));
      watch(false);
      await sleep(500);
      if (kind === 'directory') {
        const entries = native.collectTimeInfoEntries().fileTimestamps.entries;
        expect(
          entries.some(
            (entry) =>
              entry.path === path.join(link, 'b.js') &&
              entry.timestamp !== undefined,
          ),
        ).toBe(true);
        expect(
          entries.some(
            (entry) =>
              entry.path === path.join(link, 'a.js') &&
              entry.timestamp !== undefined,
          ),
        ).toBe(false);
      }
      for (const set of changes) set.clear();
      fs.writeFileSync(path.join(targets[0], 'index.js'), 'old target');
      await sleep(500);
      for (const set of changes) expect([...set]).toEqual([]);
      fs.writeFileSync(path.join(targets[1], 'index.js'), 'new target');
      await waitFor(() => changes.every((set) => set.has(link)));
      watch(false, true);
      await sleep(500);
      for (const set of changes) set.clear();
      fs.writeFileSync(path.join(targets[1], 'index.js'), 'unwatched target');
      await sleep(500);
      for (const set of changes) expect([...set]).toEqual([]);
      expect(error).toBeUndefined();
    } finally {
      watchpack.close();
      await native.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
