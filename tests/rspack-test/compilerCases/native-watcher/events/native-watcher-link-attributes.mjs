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
export default process.platform === 'linux'
  ? [false, true].map((native) => ({
      name: native ? 'native' : 'watchpack',
      description:
        'should ignore link ownership events while detecting link replacements',
      async run() {
        const root = fs.mkdtempSync(
          path.join(
            fs.realpathSync.native(os.tmpdir()),
            'rspack-link-own-attributes-',
          ),
        );
        const app = path.join(root, 'app');
        const outside = path.join(root, 'outside');
        fs.mkdirSync(app);
        fs.mkdirSync(outside);
        const past = new Date(Date.now() - 60_000);
        const targets = ['a.js', 'b.js'].map((name) => {
          const file = path.join(outside, name);
          fs.writeFileSync(file, name);
          fs.utimesSync(file, past, past);
          return file;
        });
        const link = path.join(app, 'link.js');
        fs.symlinkSync(targets[0], link);
        await sleep(2100);
        const options = { followSymlinks: true, aggregateTimeout: 50 };
        const watcher = native
          ? new NativeWatcher(options)
          : new Watchpack(options);
        const changes = new Set();
        let error;
        try {
          if (native)
            watcher.watch(
              [[link], []],
              [[], []],
              [[], []],
              BigInt(Date.now()),
              (err, batch) => {
                if (err) error = err;
                else for (const file of batch.changedFiles) changes.add(file);
              },
              () => {},
            );
          else {
            watcher.on('aggregated', (files) => {
              for (const file of files) changes.add(file);
            });
            watcher.watch({ files: [link], startTime: Date.now() });
          }
          await sleep(500);
          changes.clear();
          const before = fs.lstatSync(link).mtimeMs;
          fs.lchownSync(link, process.getuid(), process.getgid());
          expect(fs.lstatSync(link).mtimeMs).toBe(before);
          await sleep(700);
          expect([...changes]).toEqual([]);
          const replacement = path.join(app, 'replacement.js');
          fs.symlinkSync(targets[1], replacement);
          fs.renameSync(replacement, link);
          const deadline = Date.now() + 5000;
          while (!changes.has(link) && Date.now() < deadline) await sleep(10);
          expect([...changes]).toEqual([link]);
          expect(error).toBeUndefined();
        } finally {
          await watcher.close();
          fs.rmSync(root, { recursive: true, force: true });
        }
      },
    }))
  : [];
