import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);
/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should record files in newly created directory junctions',
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync.native(os.tmpdir()), 'rspack-junction-create-'),
    );
    const context = path.join(root, 'app');
    const outside = path.join(root, 'outside');
    fs.mkdirSync(context);
    fs.mkdirSync(outside);
    const target = path.join(outside, 'value.js');
    fs.writeFileSync(target, 'initial');
    const past = new Date(Date.now() - 60_000);
    fs.utimesSync(target, past, past);
    await sleep(2100);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const watcher = new NativeWatcher(options);
    let error;
    const changed = new Set();
    try {
      watcher.watch(
        [[], []],
        [[context], []],
        [[], []],
        BigInt(Date.now()),
        (err, batch) => {
          if (err) error = err;
          else for (const file of batch.changedFiles) changed.add(file);
        },
        () => {},
      );
      await sleep(500);
      changed.clear();
      const expected = [];
      for (let index = 0; index < 256; index++) {
        const link = path.join(context, `link-${index}`);
        fs.symlinkSync(outside, link, 'junction');
        expected.push(path.join(link, 'value.js'));
      }
      const missing = () => {
        const files = new Map();
        for (const entry of watcher.collectTimeInfoEntries().fileTimestamps
          .entries)
          files.set(entry.path, entry);
        return expected.filter((file) => !files.has(file));
      };
      const deadline = Date.now() + 5000;
      while (
        (missing().length || !changed.has(context)) &&
        Date.now() < deadline
      )
        await sleep(25);
      expect(missing()).toEqual([]);
      expect(changed.has(context)).toBe(true);
      expect(error).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
};
