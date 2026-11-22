import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import {
  NativeWatcher,
  Watchpack,
  waitFor,
  createEventCollector,
  watchInitial,
} from '../_helpers/watcher.mjs';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
  [
    'target-file',
    'target-tree',
    'target-directory',
    'alias',
    'no-ignore',
    'startup-target-file',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description:
      'should apply ignored to the target of a file link discovered in a context',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-ignored-context-file-links-',
        ),
      );
      const app = path.join(root, 'app');
      const outside = path.join(root, 'outside');
      const target = path.join(outside, 'value.js');
      const link = path.join(app, 'link.js');
      const witness = path.join(app, 'witness.js');
      const startup = scenario === 'startup-target-file';
      fs.mkdirSync(app);
      fs.mkdirSync(outside);
      fs.writeFileSync(target, 'before');
      fs.writeFileSync(witness, 'before');
      const old = new Date(Date.now() - 60_000);
      fs.utimesSync(target, old, old);
      fs.utimesSync(witness, old, old);
      fs.symlinkSync(target, link, 'file');
      await sleep(2200);
      const startTime = Date.now();
      if (startup) {
        await sleep(50);
        fs.writeFileSync(target, 'changed during build');
        await sleep(500);
      }
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored(entry) {
          switch (scenario) {
            case 'target-file':
            case 'startup-target-file':
              return entry === target;
            case 'target-tree':
              return (
                entry === outside || entry.startsWith(`${outside}${path.sep}`)
              );
            case 'target-directory':
              return entry === outside;
            case 'alias':
              return entry === link;
            default:
              return false;
          }
        },
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const collected = createEventCollector();
      const { changes, removals } = collected;
      try {
        watchInitial(watcher, { directories: [app], startTime }, collected);
        await waitFor(() => {
          const entries = native
            ? watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.map((entry) => entry.path)
            : [...watcher.getTimeInfoEntries().keys()];
          return entries.includes(witness);
        });
        await sleep(500);
        if (!startup) {
          changes.clear();
          removals.clear();
          fs.writeFileSync(target, 'live edit');
        }
        const expected =
          scenario === 'target-directory' || scenario === 'no-ignore'
            ? [app]
            : [];
        if (expected.length) await waitFor(() => changes.has(app));
        await sleep(500);
        expect([...changes]).toEqual(expected);
        expect([...removals]).toEqual([]);
        if (scenario === 'target-file' || scenario === 'target-tree') {
          fs.unlinkSync(link);
          fs.symlinkSync(target, link, 'file');
          await waitFor(() => changes.has(app));
        }
        changes.clear();
        fs.writeFileSync(witness, 'ordinary edit');
        await waitFor(() => changes.has(app));
        expect(collected.error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
