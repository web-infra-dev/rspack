import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  'visible',
  'ignored-alias-file',
  'ignored-target-file',
  'ignored-target-directory',
  'ignored-parent',
  'retarget-once',
  'create-link',
].map((scenario) => ({
  name: `native-${scenario}`,
  description:
    'should retain visible parent ownership of a recovered or retargeted context root',
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync.native(os.tmpdir()), 'rspack-context-root-'),
    );
    const app = path.join(root, 'app');
    const container = path.join(app, 'container');
    const link = path.join(container, 'link');
    const alias = path.join(link, 'value.js');
    const witness = path.join(app, 'witness.js');
    const targets = ['a', 'b'].map((name) => path.join(root, name));
    const old = new Date(Date.now() - 60_000);
    fs.mkdirSync(container, { recursive: true });
    fs.writeFileSync(witness, 'before');
    fs.utimesSync(witness, old, old);
    for (const target of targets) {
      fs.mkdirSync(target);
      const file = path.join(target, 'value.js');
      fs.writeFileSync(file, 'before');
      fs.utimesSync(file, old, old);
    }
    if (scenario !== 'create-link')
      fs.symlinkSync(targets[0], link, 'junction');
    await sleep(2200);

    const ignoredParent = scenario === 'ignored-parent';
    const ignoredAlias = scenario === 'ignored-alias-file';
    const options = {
      followSymlinks: true,
      aggregateTimeout: 50,
      ignored: (entry) =>
        (ignoredParent && entry === container) ||
        (ignoredAlias && entry === alias) ||
        (scenario === 'ignored-target-file' &&
          targets.some((target) => entry === path.join(target, 'value.js'))) ||
        (scenario === 'ignored-target-directory' && targets.includes(entry)),
    };
    const watcher = new NativeWatcher(options);
    const changes = new Set();
    const immediate = new Set();
    let failure;
    const record = (changed) => {
      for (const entry of changed) changes.add(entry);
    };
    const watch = (initial, remove = false) => {
      const startTime = Date.now();
      watcher.watch(
        [[], []],
        [initial ? [app, link] : [], remove ? [link] : []],
        [[], []],
        BigInt(startTime),
        (error, batch) => {
          if (error) failure = error;
          else record(batch.changedFiles);
        },
        (event) => immediate.add(event.path),
      );
    };
    const entries = () =>
      watcher
        .collectTimeInfoEntries()
        .fileTimestamps.entries.map((entry) => entry.path);
    const parentTime = () =>
      watcher
        .collectTimeInfoEntries()
        .directoryTimestamps.entries.find((entry) => entry.path === app)
        ?.safeTime;
    const waitFor = async (predicate) => {
      const deadline = Date.now() + 5000;
      while (!predicate() && Date.now() < deadline) await sleep(10);
      expect(predicate()).toBe(true);
    };
    try {
      watch(true);
      await waitFor(() => entries().includes(witness));
      await sleep(400);
      const nextTargets =
        scenario === 'create-link'
          ? [targets[0]]
          : scenario === 'retarget-once'
            ? [targets[1]]
            : [targets[1], targets[0]];
      for (const target of nextTargets) {
        changes.clear();
        const replacement = path.join(container, 'replacement');
        fs.symlinkSync(target, replacement, 'junction');
        if (process.platform === 'win32' && fs.existsSync(link))
          fs.unlinkSync(link);
        fs.renameSync(replacement, link);
        await waitFor(() => changes.has(link));
        watch(false);
        await sleep(400);
      }
      watch(false, true);
      await sleep(400);
      expect(entries().includes(link)).toBe(!ignoredParent);
      changes.clear();
      immediate.clear();
      const before = parentTime();
      expect(typeof before).toBe('number');
      fs.writeFileSync(
        path.join(nextTargets.at(-1), 'value.js'),
        'after removing the explicit context',
      );
      const ignored = ignoredParent || ignoredAlias;
      if (!ignored) await waitFor(() => changes.has(app));
      await sleep(500);
      expect([...changes]).toEqual(ignored ? [] : [app]);
      expect([...immediate]).toEqual(ignored ? [] : [alias]);
      expect(entries().includes(alias)).toBe(!ignored);
      if (ignored) expect(parentTime()).toBe(before);
      else expect(parentTime()).toBeGreaterThan(before);
      expect(failure).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
