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
const spellings =
  process.platform === 'win32'
    ? ['original', 'canonical', 'lowercase']
    : ['canonical'];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default spellings.flatMap((spelling) =>
  [false, true].map((directory) => ({
    name: `${spelling}-${directory ? 'directory' : 'file'}`,
    description:
      'should map physical target events back to the registered path spelling',
    async run() {
      if (process.env.RSPACK_WATCHER_TRACE && !tracing) {
        await require('@rspack/core').experiments.globalTrace.register(
          'rspack_watcher::fs_event=debug',
          'logger',
          'stdout',
        );
        tracing = true;
      }
      const original = fs.mkdtempSync(
        path.join(os.tmpdir(), 'rspack-path-identity-'),
      );
      const canonical = fs.realpathSync.native(original);
      const root =
        spelling === 'original'
          ? original
          : spelling === 'lowercase'
            ? canonical.toLowerCase()
            : canonical;
      const source = path.join(root, 'app');
      const target = path.join(root, 'target');
      fs.mkdirSync(source);
      fs.mkdirSync(target);
      const file = path.join(target, 'index.js');
      fs.writeFileSync(file, 'initial');
      const past = new Date(Date.now() - 60_000);
      fs.utimesSync(file, past, past);
      const link = path.join(source, 'link');
      fs.symlinkSync(
        directory ? target : file,
        link,
        directory ? 'junction' : 'file',
      );
      const options = { followSymlinks: true, aggregateTimeout: 50 };
      const native = new NativeWatcher(options);
      const watchpack = new Watchpack(options);
      const nativeChanges = new Set();
      const watchpackChanges = new Set();
      let error;
      try {
        if (process.env.RSPACK_WATCHER_TRACE)
          console.log(
            'PATH_IDENTITY_INPUT',
            JSON.stringify({
              spelling,
              directory,
              original,
              canonical,
              link,
              target: fs.readlinkSync(link),
              realTarget: fs.realpathSync.native(link),
            }),
          );
        await sleep(2100);
        native.watch(
          [directory ? [] : [link], []],
          [directory ? [link] : [], []],
          [[], []],
          BigInt(Date.now()),
          (err, batch) => {
            if (err) error = err;
            else
              for (const changed of batch.changedFiles)
                nativeChanges.add(changed);
          },
          () => {},
        );
        watchpack.on('aggregated', (changes) => {
          for (const changed of changes) watchpackChanges.add(changed);
        });
        watchpack.watch({
          files: directory ? [] : [link],
          directories: directory ? [link] : [],
          startTime: Date.now(),
        });
        await sleep(500);
        nativeChanges.clear();
        watchpackChanges.clear();
        fs.writeFileSync(file, 'changed');
        const deadline = Date.now() + 5000;
        while (
          (!nativeChanges.has(link) || !watchpackChanges.has(link)) &&
          Date.now() < deadline
        )
          await sleep(10);
        if (process.env.RSPACK_WATCHER_TRACE)
          console.log(
            'PATH_IDENTITY_RESULT',
            JSON.stringify({
              spelling,
              directory,
              expected: link,
              native: [...nativeChanges],
              watchpack: [...watchpackChanges],
              error: error?.message,
            }),
          );
        expect(error).toBeUndefined();
        expect([...watchpackChanges]).toEqual([link]);
        expect([...nativeChanges]).toEqual([link]);
      } finally {
        await native.close();
        watchpack.close();
        fs.rmSync(original, { recursive: true, force: true });
      }
    },
  })),
);
