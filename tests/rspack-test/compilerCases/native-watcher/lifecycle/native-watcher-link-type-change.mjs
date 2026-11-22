import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
import {
  NativeWatcher,
  Watchpack,
  createEventCollector,
} from '../_helpers/watcher.mjs';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default process.platform === 'win32'
  ? []
  : [false, true].flatMap((native) =>
      [false, true].map((directory) => ({
        name: `${native ? 'native' : 'watchpack'}-${directory ? 'directory-to-file' : 'file-to-directory'}`,
        description:
          'should invalidate a dependency when its link changes target type',
        async run() {
          const root = fs.mkdtempSync(
            path.join(fs.realpathSync.native(os.tmpdir()), 'rspack-link-type-'),
          );
          const app = path.join(root, 'app');
          const targetDirectory = path.join(root, 'directory');
          const targetFile = path.join(root, 'file.js');
          fs.mkdirSync(app);
          fs.mkdirSync(targetDirectory);
          fs.writeFileSync(targetFile, 'initial');
          const link = path.join(app, 'link');
          fs.symlinkSync(directory ? targetDirectory : targetFile, link);
          await sleep(2100);
          const options = { followSymlinks: true, aggregateTimeout: 50 };
          const watcher = native
            ? new NativeWatcher(options)
            : new Watchpack(options);
          const collected = createEventCollector();
          const { changes, removals, record, callback } = collected;
          try {
            if (native)
              watcher.watch(
                [directory ? [] : [link], []],
                [directory ? [link] : [], []],
                [[], []],
                BigInt(Date.now()),
                callback,
                () => {},
              );
            else {
              watcher.on('aggregated', record);
              watcher.watch({
                files: directory ? [] : [link],
                directories: directory ? [link] : [],
                startTime: Date.now(),
              });
            }
            await sleep(500);
            changes.clear();
            removals.clear();
            const replacement = path.join(app, 'replacement');
            fs.symlinkSync(
              directory ? targetFile : targetDirectory,
              replacement,
            );
            fs.renameSync(replacement, link);
            const deadline = Date.now() + 5000;
            while (!changes.has(link) && Date.now() < deadline) await sleep(10);
            expect([...changes]).toEqual([link]);
            expect([...removals]).toEqual([]);
            expect(collected.error).toBeUndefined();
          } finally {
            await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      })),
    );
