import assert from 'node:assert/strict';
import { once } from 'node:events';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';

export default async function run() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-watchpack-'));
  const removed = path.join(root, 'dep.js');
  const alreadyAbsent = path.join(root, 'absent.js');
  const optional = path.join(root, 'optional.js');
  const removeBeforeStat = new Set([removed, optional]);
  const lstat = fs.lstat;
  let watcher;
  try {
    for (const file of removeBeforeStat) fs.writeFileSync(file, '');
    fs.lstat = (file, ...args) => {
      if (removeBeforeStat.delete(file)) fs.unlinkSync(file);
      return lstat(file, ...args);
    };
    const require = createRequire(import.meta.url);
    const coreDir = path.dirname(require.resolve('@rspack/core/package.json'));
    const Watchpack = require(path.join(coreDir, 'compiled/watchpack'));
    watcher = new Watchpack({ poll: true, aggregateTimeout: 0 });
    const batch = once(watcher, 'aggregated');
    // The already absent file guarantees a batch even if the deletion is lost.
    watcher.watch({
      files: [removed, alreadyAbsent],
      missing: [optional],
      startTime: Date.now(),
    });
    const [, removals] = await batch;
    assert.equal(removeBeforeStat.size, 0);
    assert.deepEqual(removals, new Set([removed, alreadyAbsent]));
  } finally {
    watcher?.close();
    fs.lstat = lstat;
    fs.rmSync(root, { recursive: true, force: true });
  }
}
