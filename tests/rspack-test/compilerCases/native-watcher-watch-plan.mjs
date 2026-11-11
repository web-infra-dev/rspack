import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

const spellings =
  process.platform === 'win32'
    ? ['original', 'canonical', 'lowercase']
    : ['canonical'];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  ...spellings.flatMap((spelling) =>
    [false, true].map((followSymlinks) => ({
      name: `plain-${spelling}-${followSymlinks ? 'follow' : 'no-follow'}`,
      description:
        'should preserve ordinary file dependencies when preparing backend paths',
      async run() {
        const original = fs.mkdtempSync(
          path.join(os.tmpdir(), 'rspack-plan-plain-'),
        );
        const canonical = fs.realpathSync.native(original);
        const root =
          spelling === 'original'
            ? original
            : spelling === 'lowercase'
              ? canonical.toLowerCase()
              : canonical;
        const file = path.join(root, 'index.js');
        fs.writeFileSync(file, 'initial');
        const past = new Date(Date.now() - 60_000);
        fs.utimesSync(file, past, past);
        const watcher = new NativeWatcher({
          followSymlinks,
          aggregateTimeout: 50,
        });
        const changes = new Set();
        let error;
        try {
          await sleep(2100);
          watcher.watch(
            [[file], []],
            [[], []],
            [[], []],
            BigInt(Date.now()),
            (err, batch) => {
              if (err) error = err;
              else
                for (const changed of batch.changedFiles) changes.add(changed);
            },
            () => {},
          );
          await sleep(500);
          changes.clear();
          fs.writeFileSync(file, 'changed');
          await waitFor(() => changes.has(file));
          expect([...changes]).toEqual([file]);
          expect(error).toBeUndefined();
        } finally {
          await watcher.close();
          fs.rmSync(original, { recursive: true, force: true });
        }
      },
    })),
  ),
  {
    name: 'merge-physical-roots',
    description:
      'should retain recursive coverage when a directory link and a file share a physical watch root',
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), 'rspack-plan-roots-'),
      );
      const target = path.join(root, 'target');
      const deepFile = path.join(target, 'nested', 'deep.js');
      const file = path.join(target, 'index.js');
      const link = path.join(root, 'link');
      fs.mkdirSync(path.dirname(deepFile), { recursive: true });
      const past = new Date(Date.now() - 60_000);
      for (const entry of [file, deepFile]) {
        fs.writeFileSync(entry, 'initial');
        fs.utimesSync(entry, past, past);
      }
      fs.symlinkSync(target, link, 'junction');
      const watcher = new NativeWatcher({
        followSymlinks: true,
        aggregateTimeout: 50,
      });
      const changes = new Set();
      let error;
      const callback = (err, batch) => {
        if (err) error = err;
        else for (const changed of batch.changedFiles) changes.add(changed);
      };
      try {
        await sleep(2100);
        watcher.watch(
          [[file], []],
          [[link], []],
          [[], []],
          BigInt(Date.now()),
          callback,
          () => {},
        );
        await sleep(500);
        changes.clear();
        fs.writeFileSync(deepFile, 'recursive change');
        await waitFor(() => changes.has(link));
        expect([...changes]).toEqual([link]);
        watcher.watch(
          [[], [file]],
          [[], []],
          [[], []],
          BigInt(Date.now()),
          callback,
          () => {},
        );
        await sleep(500);
        changes.clear();
        fs.writeFileSync(deepFile, 'after removing the file dependency');
        await waitFor(() => changes.has(link));
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  },
];
