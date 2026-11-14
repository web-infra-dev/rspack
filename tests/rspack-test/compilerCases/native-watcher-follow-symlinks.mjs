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

const scenarios = [
  'file-target',
  'target-roundtrip',
  'file-chain',
  'parent-links',
  'directory-target',
  'watched-directory',
  'multiple-aliases',
  'nested-file-target',
  'nested-file-inside',
  'nested-directory-target',
  'nested-directory-ignored-real',
  'nested-directory-cycle',
  'missing-target',
  'missing-directory',
  'retarget-chain',
  'retarget-direct',
  'ignored-target',
  'ignored-link',
  'no-follow',
];

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default scenarios.map((scenario) => ({
  name: scenario,
  description: `should follow symbolic links for ${scenario}`,
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-links-'),
    );
    const source = path.join(root, 'app');
    const outside = path.join(root, 'lib');
    fs.mkdirSync(source);
    fs.mkdirSync(outside);
    const target =
      scenario === 'nested-file-inside'
        ? path.join(source, 'index.js')
        : path.join(outside, 'dist', 'index.js');
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, 'before');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(target, past, past);
    const link = path.join(source, 'link');
    const directory =
      scenario.includes('directory') || scenario === 'parent-links';
    const nested = scenario.startsWith('nested');
    const missing = scenario.startsWith('missing');
    const chain = scenario.includes('chain');
    const watchedDirectory = scenario === 'watched-directory';
    const intermediate = path.join(outside, 'intermediate');
    if (scenario === 'parent-links') {
      fs.renameSync(target, path.join(outside, 'value.js'));
      fs.symlinkSync('../value.js', target, 'file');
    }
    if (chain) fs.symlinkSync('dist/index.js', intermediate, 'file');
    fs.symlinkSync(
      chain ? intermediate : directory ? outside : target,
      link,
      directory ? 'junction' : 'file',
    );
    const dependency = directory ? path.join(link, 'dist', 'index.js') : link;
    if (scenario === 'nested-directory-cycle')
      fs.symlinkSync(source, path.join(source, 'cycle'), 'junction');
    const secondLink = path.join(source, 'second-link');
    if (scenario === 'multiple-aliases')
      fs.symlinkSync(target, secondLink, 'file');
    if (missing) fs.rmSync(path.dirname(target), { recursive: true });
    // Avoid both startup mtime backfill and stale FSEvents masking missing watches.
    await sleep(2100);
    const files =
      nested || missing || watchedDirectory
        ? []
        : [
            dependency,
            ...(scenario === 'multiple-aliases' ? [secondLink] : []),
          ];
    const directories = nested ? [source] : watchedDirectory ? [link] : [];
    const missingFiles = missing ? [dependency] : [];
    const options = {
      aggregateTimeout: 50,
      followSymlinks: scenario !== 'no-follow',
      // Like Watchpack, inspect the target after the synchronous rename pair.
      ...(scenario === 'target-roundtrip' ? { ignored: () => false } : {}),
      ...(scenario.includes('ignored')
        ? {
            ignored: (scenario === 'ignored-link' ? link : outside).replaceAll(
              '\\',
              '/',
            ),
          }
        : {}),
    };
    const native = new NativeWatcher(options);
    const watchpack = new Watchpack(options);
    const nativeChanges = new Set();
    const watchpackChanges = new Set();
    const nativeRemovals = new Set();
    const watchpackRemovals = new Set();
    const nativeEvents = [];
    const watchpackEvents = [];
    let nativeError;
    native.watch(
      [files, []],
      [directories, []],
      [missingFiles, []],
      BigInt(Date.now()),
      (error, batch) => {
        if (error) nativeError = error;
        else {
          for (const file of batch.changedFiles) nativeChanges.add(file);
          for (const file of batch.removedFiles) nativeRemovals.add(file);
        }
      },
      (event) => nativeEvents.push(event.path),
    );
    watchpack.on('change', (file) => watchpackEvents.push(file));
    watchpack.on('remove', (file) => watchpackEvents.push(file));
    watchpack.on('aggregated', (changes, removals) => {
      for (const file of changes) watchpackChanges.add(file);
      for (const file of removals) watchpackRemovals.add(file);
    });
    watchpack.watch({
      files,
      directories,
      missing: missingFiles,
      startTime: Date.now(),
    });
    try {
      await sleep(500);
      nativeChanges.clear();
      watchpackChanges.clear();
      nativeEvents.length = watchpackEvents.length = 0;
      const changedAt = Date.now();
      if (scenario.startsWith('retarget')) {
        const replacement = path.join(outside, 'replacement.js');
        fs.writeFileSync(replacement, 'after');
        fs.utimesSync(replacement, past, past);
        const changedLink = chain ? intermediate : link;
        fs.unlinkSync(changedLink);
        fs.symlinkSync(replacement, changedLink, 'file');
      } else if (scenario === 'target-roundtrip') {
        fs.renameSync(target, `${target}.bak`);
        fs.renameSync(`${target}.bak`, target);
      } else {
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, 'after');
      }
      const expected = nested ? source : watchedDirectory ? link : dependency;
      if (
        scenario.startsWith('ignored') ||
        scenario === 'no-follow' ||
        scenario === 'target-roundtrip'
      ) {
        await sleep(600);
        expect([...watchpackChanges]).toEqual([]);
        expect([...nativeChanges]).toEqual([]);
        return;
      }
      await waitFor(() => watchpackChanges.has(expected));
      await waitFor(() => nativeChanges.has(expected));
      expect(nativeError).toBeUndefined();
      if (scenario === 'multiple-aliases') {
        await waitFor(
          () =>
            nativeChanges.has(secondLink) && watchpackChanges.has(secondLink),
        );
      }
      expect([...nativeChanges].sort()).toEqual([...watchpackChanges].sort());
      if (scenario.startsWith('retarget')) {
        const entry = native
          .collectTimeInfoEntries()
          .fileTimestamps.entries.find((entry) => entry.path === dependency);
        expect(entry.safeTime).toBeGreaterThanOrEqual(changedAt);
        expect(entry.timestamp).toBe(past.getTime());
      }
      if (!scenario.startsWith('retarget')) {
        const eventPath =
          watchedDirectory || scenario === 'nested-file-inside'
            ? target
            : dependency;
        if (scenario === 'nested-file-inside') {
          expect([...new Set(nativeEvents)].sort()).toEqual(
            [...new Set(watchpackEvents)].sort(),
          );
          return;
        }
        expect(nativeEvents).toContain(eventPath);
        expect(watchpackEvents).toContain(eventPath);
        const entry = native
          .collectTimeInfoEntries()
          .fileTimestamps.entries.find((entry) => entry.path === dependency);
        expect(entry.timestamp).toBe(Math.round(fs.statSync(target).mtimeMs));
        for (const set of [
          nativeChanges,
          watchpackChanges,
          nativeRemovals,
          watchpackRemovals,
        ])
          set.clear();
        nativeEvents.length = watchpackEvents.length = 0;
        fs.unlinkSync(target);
        if (scenario === 'nested-file-target') {
          await sleep(600);
          expect([...watchpackChanges]).toEqual([]);
          expect([...nativeChanges]).toEqual([]);
          fs.writeFileSync(target, 'restored target');
          await waitFor(() =>
            [nativeChanges, watchpackChanges].every((set) => set.has(expected)),
          );
          return;
        }
        const removed =
          nested || watchedDirectory
            ? [nativeChanges, watchpackChanges]
            : [nativeRemovals, watchpackRemovals];
        await waitFor(() => removed.every((set) => set.has(expected)));
        expect(nativeEvents).toContain(eventPath);
        expect(watchpackEvents).toContain(eventPath);
      }
    } finally {
      watchpack.close();
      await native.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
