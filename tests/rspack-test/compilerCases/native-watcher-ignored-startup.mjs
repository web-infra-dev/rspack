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
export default [false, true].flatMap((native) =>
  [
    'directory-target',
    'file-target',
    'missing-target',
    'descendant-target',
    'directory-link',
    'file-link',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description:
      'should backfill only retained link subscriptions when targets are ignored',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-ignored-startup-',
        ),
      );
      const outside = path.join(root, 'outside');
      const app = path.join(root, 'app');
      fs.mkdirSync(outside);
      fs.mkdirSync(app);
      const targetFile = path.join(outside, 'entry.js');
      const directory = scenario.startsWith('directory');
      const descendant = scenario === 'descendant-target';
      const missing = scenario === 'missing-target';
      const replaceLink = scenario.endsWith('-link');
      if (!missing) {
        fs.writeFileSync(targetFile, 'initial');
        const past = new Date(Date.now() - 60_000);
        fs.utimesSync(targetFile, past, past);
      }
      const link = path.join(app, 'link');
      const linkTarget = directory || descendant ? outside : targetFile;
      const linkType = directory || descendant ? 'junction' : 'file';
      fs.symlinkSync(linkTarget, link, linkType);
      const registered = descendant ? path.join(link, 'entry.js') : link;
      await sleep(2100);
      const startTime = Date.now();
      await sleep(25);
      if (replaceLink) {
        fs.unlinkSync(link);
        fs.symlinkSync(linkTarget, link, linkType);
      } else if (directory)
        fs.writeFileSync(path.join(outside, 'new.js'), 'new');
      else fs.writeFileSync(targetFile, 'changed');
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored: (file) =>
          file === outside || file.startsWith(`${outside}${path.sep}`),
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changes = new Set();
      const removals = new Set();
      let error;
      const record = (changed, removed) => {
        for (const file of changed) changes.add(file);
        for (const file of removed) removals.add(file);
      };
      try {
        if (native)
          watcher.watch(
            [!directory && !missing ? [registered] : [], []],
            [directory ? [registered] : [], []],
            [missing ? [registered] : [], []],
            BigInt(startTime),
            (err, batch) => {
              if (err) error = err;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        else {
          watcher.on('aggregated', record);
          watcher.watch({
            files: !directory && !missing ? [registered] : [],
            directories: directory ? [registered] : [],
            missing: missing ? [registered] : [],
            startTime,
          });
        }
        if (replaceLink) {
          const deadline = Date.now() + 5000;
          while (!changes.has(registered) && Date.now() < deadline)
            await sleep(10);
        } else await sleep(700);
        expect([...changes]).toEqual(replaceLink ? [registered] : []);
        expect([...removals]).toEqual([]);
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
