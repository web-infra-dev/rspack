import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { NativeWatcher, waitFor } from '../_helpers/watcher.mjs';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  'visible',
  'ignored-target-file',
  'ignored-target-directory',
].map((scenario) => ({
  name: `native-${scenario}`,
  description:
    'should update discovered link types and apply ignored to a replacement file target',
  async run() {
    const root = fs.mkdtempSync(
      path.join(
        fs.realpathSync.native(os.tmpdir()),
        'rspack-context-link-type-',
      ),
    );
    const app = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    const directory = path.join(outside, 'directory');
    const child = path.join(directory, 'child.js');
    const target = path.join(outside, 'value.js');
    const link = path.join(app, 'link');
    const aliasChild = path.join(link, 'child.js');
    const witness = path.join(app, 'witness.js');
    fs.mkdirSync(app);
    fs.mkdirSync(directory, { recursive: true });
    const old = new Date(Date.now() - 60_000);
    for (const file of [child, target, witness]) {
      fs.writeFileSync(file, 'before');
      fs.utimesSync(file, old, old);
    }
    fs.symlinkSync(directory, link, 'junction');
    await sleep(2200);

    const watcher = new NativeWatcher({
      followSymlinks: true,
      aggregateTimeout: 50,
      ignored: (entry) =>
        (scenario === 'ignored-target-file' && entry === target) ||
        (scenario === 'ignored-target-directory' && entry === directory),
    });
    const changes = new Set();
    const immediate = new Set();
    let failure;
    const watch = (initial = false) => {
      watcher.watch(
        [[], []],
        [initial ? [app] : [], []],
        [[], []],
        BigInt(Date.now()),
        (error, batch) => {
          if (error) failure = error;
          else for (const entry of batch.changedFiles) changes.add(entry);
        },
        (event) => immediate.add(event.path),
      );
    };
    const entries = () =>
      watcher
        .collectTimeInfoEntries()
        .fileTimestamps.entries.map((entry) => entry.path);
    try {
      watch(true);
      await waitFor(() => entries().includes(aliasChild));
      await sleep(400);
      changes.clear();
      const replacement = path.join(root, 'replacement');
      fs.symlinkSync(target, replacement, 'file');
      if (process.platform === 'win32') fs.unlinkSync(link);
      fs.renameSync(replacement, link);
      await waitFor(() => changes.has(app));

      for (let cycle = 0; cycle < 2; cycle++) {
        watch();
        await sleep(500);
        expect(entries()).not.toContain(aliasChild);
        expect(entries()).toContain(link);
        changes.clear();
        immediate.clear();
        fs.writeFileSync(target, `file edit ${cycle}`);
        const ignored = scenario === 'ignored-target-file';
        if (!ignored) await waitFor(() => changes.has(app));
        await sleep(500);
        expect([...changes]).toEqual(ignored ? [] : [app]);
        expect([...immediate]).toEqual(ignored ? [] : [link]);
      }

      changes.clear();
      fs.writeFileSync(child, 'old directory target edit');
      await sleep(500);
      expect([...changes]).toEqual([]);
      fs.writeFileSync(witness, 'ordinary edit');
      await waitFor(() => changes.has(app));

      changes.clear();
      fs.symlinkSync(directory, replacement, 'junction');
      if (process.platform === 'win32') fs.unlinkSync(link);
      fs.renameSync(replacement, link);
      await waitFor(() => changes.has(app));
      watch();
      await waitFor(() => entries().includes(aliasChild));
      await sleep(500);
      changes.clear();
      fs.writeFileSync(child, 'restored directory target edit');
      await waitFor(() => changes.has(app));
      expect(failure).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
