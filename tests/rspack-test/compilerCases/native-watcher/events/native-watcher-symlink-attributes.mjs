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
      ['directory', 'file'].map((scenario) => ({
        name: `${native ? 'native' : 'watchpack'}-${scenario}`,
        description:
          'should ignore attribute-only changes of linked context files',
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
              'rspack-link-attributes-',
            ),
          );
          const app = path.join(root, 'app');
          const outside = path.join(root, 'outside');
          fs.mkdirSync(app);
          fs.mkdirSync(outside);
          const target = path.join(outside, 'value.js');
          fs.writeFileSync(target, 'initial');
          const past = new Date(Date.now() - 60_000);
          fs.utimesSync(target, past, past);
          const link = path.join(app, 'link');
          fs.symlinkSync(scenario === 'directory' ? outside : target, link);
          const context = scenario === 'directory' ? link : app;
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
                [[], []],
                [[context], []],
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
              watcher.watch({ directories: [context], startTime: Date.now() });
            }
            await sleep(500);
            changes.clear();
            fs.chmodSync(target, 0o600);
            await sleep(700);
            expect([...changes]).toEqual([]);
            fs.writeFileSync(target, 'content changed');
            const deadline = Date.now() + 5000;
            while (!changes.has(context) && Date.now() < deadline)
              await sleep(10);
            expect([...changes]).toEqual([context]);
            expect(error).toBeUndefined();
          } finally {
            await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      })),
    );
