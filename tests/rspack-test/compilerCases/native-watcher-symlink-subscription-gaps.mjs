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
export default [
  'intermediate-parent',
  'directory-edit',
  'directory-remove',
  'file-edit',
].map((scenario) => ({
  name: scenario,
  description: `should preserve symbolic link notifications for ${scenario}`,
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-link-gaps-'),
    );
    const context = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    const links = path.join(root, 'links');
    for (const dir of [context, outside, links]) fs.mkdirSync(dir);
    const target = path.join(outside, 'value.js');
    fs.writeFileSync(target, 'initial');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(target, past, past);
    const link = path.join(context, 'link');
    const intermediate = scenario === 'intermediate-parent';
    if (intermediate) {
      const bridge = path.join(links, 'bridge.js');
      fs.symlinkSync(target, bridge, 'file');
      fs.symlinkSync(bridge, link, 'file');
    }
    await sleep(2100);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const watcher = new NativeWatcher(options);
    const watchpack = intermediate ? new Watchpack(options) : undefined;
    const changes = new Set();
    const removals = new Set();
    const referenceRemovals = new Set();
    let error;
    const watch = (initial, startTime = Date.now()) => {
      watcher.watch(
        [initial && intermediate ? [link] : [], []],
        [initial && !intermediate ? [context] : [], []],
        [[], []],
        BigInt(startTime),
        (err, batch) => {
          if (err) error = err;
          else {
            for (const file of batch.changedFiles) changes.add(file);
            for (const file of batch.removedFiles) removals.add(file);
          }
        },
        () => {},
      );
    };
    watchpack?.on('aggregated', (_changed, removed) => {
      for (const file of removed) referenceRemovals.add(file);
    });
    watchpack?.watch({ files: [link], startTime: Date.now() });
    watch(true);
    try {
      await sleep(500);
      changes.clear();
      removals.clear();
      if (intermediate) {
        fs.chmodSync(links, 0o750);
        await sleep(300);
        expect([...changes]).toEqual([]);
        expect([...removals]).toEqual([]);
        const moved = path.join(root, 'links-old');
        fs.renameSync(links, moved);
        expect(fs.existsSync(target)).toBe(true);
        expect(fs.existsSync(link)).toBe(false);
        await waitFor(() => referenceRemovals.has(link));
        await waitFor(() => removals.has(link));
        watch(false);
        await sleep(500);
        changes.clear();
        fs.renameSync(moved, links);
        await waitFor(() => changes.has(link));
      } else {
        const fileLink = scenario === 'file-edit';
        fs.symlinkSync(
          fileLink ? target : outside,
          link,
          fileLink ? 'file' : 'junction',
        );
        await waitFor(() => changes.has(context));
        const recordedPath = fileLink ? link : path.join(link, 'value.js');
        expect(
          watcher.collectTimeInfoEntries().fileTimestamps.entries.some(
            (entry) => entry.path === recordedPath,
          ),
        ).toBe(true);
        const startTime = Date.now();
        await sleep(25);
        if (scenario === 'directory-remove') fs.unlinkSync(target);
        else fs.writeFileSync(target, 'after the build read it');
        await sleep(500);
        changes.clear();
        watch(false, startTime);
        await waitFor(() => changes.has(context));
      }
      expect(error).toBeUndefined();
    } finally {
      watchpack?.close();
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
