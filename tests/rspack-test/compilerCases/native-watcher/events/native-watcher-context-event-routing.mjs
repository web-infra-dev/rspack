import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import { NativeWatcher, Watchpack, waitFor } from '../_helpers/watcher.mjs';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].flatMap((native) =>
  [
    'visible',
    'ignored-alias',
    'ignored-target',
    'ignored-both',
    'retain-parent',
    'remove-visible-owner',
    'remove-restore',
  ].map((scenario) => ({
    name: `${native ? 'native' : 'watchpack'}-${scenario}`,
    description:
      'should route immediate events and directory times through the same visible subscriptions',
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          'rspack-context-routing-',
        ),
      );
      const app = path.join(root, 'app');
      const target = path.join(root, 'target');
      const link = path.join(app, 'link');
      const real = path.join(target, 'value.js');
      const alias = path.join(link, 'value.js');
      const witness = path.join(app, 'witness.js');
      fs.mkdirSync(app);
      fs.mkdirSync(target);
      fs.writeFileSync(real, 'before');
      fs.writeFileSync(witness, 'witness');
      const old = new Date(Date.now() - 60_000);
      fs.utimesSync(real, old, old);
      fs.utimesSync(witness, old, old);
      fs.symlinkSync(target, link, 'junction');
      await sleep(2200);

      const ignoreAlias = [
        'ignored-alias',
        'ignored-both',
        'remove-visible-owner',
      ].includes(scenario);
      const ignoreTarget =
        scenario === 'ignored-target' || scenario === 'ignored-both';
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored: (entry) =>
          (ignoreAlias && entry === alias) || (ignoreTarget && entry === real),
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const changes = new Set();
      const immediate = new Set();
      let failure;
      const record = (changed) => {
        for (const entry of changed) changes.add(entry);
      };
      const aggregate = (error, batch) => {
        if (error) failure = error;
        else record(batch.changedFiles);
      };
      const recordImmediate = (entry) => {
        if (entry === alias || entry === real) immediate.add(entry);
      };
      const parentTime = () => {
        if (native)
          return watcher
            .collectTimeInfoEntries()
            .directoryTimestamps.entries.find((entry) => entry.path === app)
            ?.safeTime;
        return watcher.getTimeInfoEntries().get(app)?.safeTime;
      };
      try {
        const startTime = Date.now();
        if (native)
          watcher.watch(
            [[], []],
            [[app, link], []],
            [[], []],
            BigInt(startTime),
            aggregate,
            (event) => recordImmediate(event.path),
          );
        else {
          watcher.on('aggregated', record);
          watcher.on('change', recordImmediate);
          watcher.watch({ directories: [app, link], startTime });
        }
        await waitFor(() => {
          const entries = native
            ? watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.map((entry) => entry.path)
            : [...watcher.getTimeInfoEntries().keys()];
          return entries.includes(witness);
        });
        await sleep(400);
        const before = parentTime();
        expect(typeof before).toBe('number');
        changes.clear();
        immediate.clear();
        fs.writeFileSync(real, 'after');
        const expectedChanges = [
          ...(!ignoreAlias ? [app] : []),
          ...(!ignoreTarget ? [link] : []),
        ];
        const expectedImmediate = [
          ...(!ignoreAlias ? [alias] : []),
          ...(!ignoreTarget ? [real] : []),
        ];
        if (expectedChanges.length)
          await waitFor(() =>
            expectedChanges.every((entry) => changes.has(entry)),
          );
        await sleep(400);
        expect([...changes].sort()).toEqual(expectedChanges.sort());
        expect([...immediate].sort()).toEqual(expectedImmediate.sort());
        if (ignoreAlias) expect(parentTime()).toBe(before);
        else expect(parentTime()).toBeGreaterThan(before);
        if (scenario === 'remove-restore') {
          for (const operation of ['remove', 'restore']) {
            changes.clear();
            immediate.clear();
            if (operation === 'remove') fs.unlinkSync(real);
            else fs.writeFileSync(real, 'restored');
            await waitFor(() => changes.has(app) && changes.has(link));
            await sleep(400);
            expect([...changes].sort()).toEqual([app, link].sort());
            expect([...immediate].sort()).toEqual([alias, real].sort());
          }
        }
        if (
          scenario === 'retain-parent' ||
          scenario === 'remove-visible-owner'
        ) {
          if (native)
            watcher.watch(
              [[], []],
              [[], [link]],
              [[], []],
              BigInt(Date.now()),
              aggregate,
              (event) => recordImmediate(event.path),
            );
          else watcher.watch({ directories: [app], startTime: Date.now() });
          await sleep(400);
          changes.clear();
          immediate.clear();
          if (ignoreAlias) {
            const entries = native
              ? watcher
                  .collectTimeInfoEntries()
                  .fileTimestamps.entries.map((entry) => entry.path)
              : [...watcher.getTimeInfoEntries().keys()];
            expect(entries).not.toContain(alias);
          }
          fs.writeFileSync(real, 'after removing the explicit context');
          if (!ignoreAlias) await waitFor(() => changes.has(app));
          await sleep(400);
          expect([...changes]).toEqual(ignoreAlias ? [] : [app]);
          expect([...immediate]).toEqual(ignoreAlias ? [] : [alias]);
        }
        expect(failure).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
