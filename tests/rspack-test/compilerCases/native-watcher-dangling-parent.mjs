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

let tracing = false;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default process.platform === 'win32'
  ? []
  : [false, true].flatMap((native) =>
      [false, true].flatMap((missing) =>
        ['outside', 'same-directory'].map((parent) => ({
          name: `${native ? 'native' : 'watchpack'}-${missing ? 'missing' : 'file'}-${parent}`,
          description:
            'should ignore parent attributes of an already dangling file link',
          async run() {
            if (native && process.env.RSPACK_WATCHER_TRACE && !tracing) {
              await require('@rspack/core').experiments.globalTrace.register(
                'rspack_watcher::fs_event=debug',
                'logger',
                'stdout',
              );
              tracing = true;
            }
            const root = fs.mkdtempSync(
              path.join(
                fs.realpathSync.native(os.tmpdir()),
                'rspack-dangling-parent-',
              ),
            );
            const app = path.join(root, 'app');
            const outside = path.join(root, 'outside');
            fs.mkdirSync(app);
            fs.mkdirSync(outside);
            const link = path.join(app, 'link.js');
            fs.symlinkSync(
              path.join(
                parent === 'same-directory' ? app : outside,
                'missing.js',
              ),
              link,
              'file',
            );
            await sleep(2100);
            const options = { followSymlinks: true, aggregateTimeout: 50 };
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
                  [missing ? [] : [link], []],
                  [[], []],
                  [missing ? [link] : [], []],
                  BigInt(Date.now()),
                  (err, batch) => {
                    if (err) error = err;
                    else record(batch.changedFiles, batch.removedFiles);
                  },
                  () => {},
                );
              else {
                watcher.on('aggregated', record);
                watcher.watch({
                  files: missing ? [] : [link],
                  missing: missing ? [link] : [],
                  startTime: Date.now(),
                });
              }
              await sleep(500);
              changes.clear();
              removals.clear();
              fs.chmodSync(app, 0o750);
              await sleep(500);
              expect([...changes]).toEqual([]);
              expect([...removals]).toEqual([]);
              expect(error).toBeUndefined();
            } finally {
              await watcher.close();
              fs.rmSync(root, { recursive: true, force: true });
            }
          },
        })),
      ),
    );
