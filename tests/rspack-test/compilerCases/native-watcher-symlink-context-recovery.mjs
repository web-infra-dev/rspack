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
export default [false, true].flatMap((native) =>
  [
    ...(native ? ['recovery-target', 'recovery-parent'] : []),
    'explicit-real',
    'explicit-alias',
    'nested-real',
    'nested-alias',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description: `should preserve directory link subscriptions and ignored paths for ${scenario}`,
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), 'rspack-context-link-'),
      );
      const source = path.join(root, 'app');
      const target = path.join(root, 'outside', 'output');
      const realFile = path.join(target, 'generated', 'index.js');
      fs.mkdirSync(source);
      fs.mkdirSync(path.dirname(realFile), { recursive: true });
      fs.writeFileSync(realFile, 'initial');
      const past = new Date(Date.now() - 60_000);
      fs.utimesSync(realFile, past, past);
      const link = path.join(source, 'link');
      fs.symlinkSync(target, link, 'junction');
      const aliasFile = path.join(link, 'generated', 'index.js');
      const explicit = scenario.startsWith('explicit');
      const context = explicit ? link : source;
      const ignoredDirectory = path.join(
        scenario.endsWith('real') ? target : link,
        'generated',
      );
      const ignoredCalls = new Set();
      const options = {
        aggregateTimeout: 50,
        followSymlinks: true,
        ignored: (file) => {
          ignoredCalls.add(file);
          return (
            !scenario.startsWith('recovery') &&
            (file === ignoredDirectory ||
              file.startsWith(`${ignoredDirectory}${path.sep}`))
          );
        },
      };
      await sleep(2100);
      let watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changes = new Set();
      const raw = new Set();
      let error;
      const watch = (initial) => {
        if (native)
          watcher.watch(
            [[], []],
            [initial ? [context] : [], []],
            [[], []],
            BigInt(Date.now()),
            (err, batch) => {
              if (err) error = err;
              else for (const file of batch.changedFiles) changes.add(file);
            },
            (event) => raw.add(event.path),
          );
        else {
          const old = watcher;
          watcher = new Watchpack(options);
          watcher.on('aggregated', (files) => {
            for (const file of files) changes.add(file);
          });
          watcher.on('change', (file) => raw.add(file));
          watcher.watch({
            files: [],
            directories: [context],
            missing: [],
            startTime: Date.now(),
          });
          // Match NodeWatchFileSystem: install the new watchers before closing the old ones.
          old.close();
        }
      };
      watch(true);
      try {
        await sleep(500);
        changes.clear();
        raw.clear();
        if (scenario.startsWith('recovery')) {
          fs.rmSync(
            scenario === 'recovery-parent' ? path.dirname(target) : target,
            { recursive: true },
          );
          await waitFor(() => changes.has(context));
          watch(false);
          await sleep(500);
          changes.clear();
          raw.clear();
          fs.mkdirSync(path.dirname(realFile), { recursive: true });
          fs.writeFileSync(realFile, 'restored');
          await waitFor(() => changes.has(context));
          watch(false);
          await sleep(500);
          changes.clear();
          raw.clear();
          fs.writeFileSync(realFile, 'after rewatch');
          await waitFor(() => raw.has(aliasFile));
          watcher.watch(
            [[], []],
            [[], [context]],
            [[], []],
            BigInt(Date.now()),
            () => {},
            (event) => raw.add(event.path),
          );
          await sleep(500);
          raw.clear();
          fs.writeFileSync(realFile, 'after unregister');
          await sleep(700);
          expect([...raw]).toEqual([]);
        } else {
          const ignored =
            scenario === 'explicit-real' || scenario === 'nested-alias';
          const timestamps = native
            ? new Set(
                watcher
                  .collectTimeInfoEntries()
                  .fileTimestamps.entries.map((entry) => entry.path),
              )
            : new Set(watcher.getTimeInfoEntries().keys());
          expect(
            timestamps.has(native || !explicit ? aliasFile : realFile),
          ).toBe(!ignored);
          expect(
            ignoredCalls.has(path.join(explicit ? target : link, 'generated')),
          ).toBe(true);
          fs.writeFileSync(realFile, 'changed');
          if (ignored) {
            await sleep(700);
            expect([...changes]).toEqual([]);
            expect([...raw]).toEqual([]);
          } else {
            await waitFor(() => changes.has(context));
            expect(raw.has(explicit ? realFile : aliasFile)).toBe(true);
          }
        }
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
