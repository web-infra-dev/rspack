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

// FSEvents canonicalizes paths even when link following is disabled.
const modes = process.platform === 'darwin' ? [true] : [false, true];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default ['file', 'directory', 'missing', 'shared-file'].flatMap((kind) =>
  (kind === 'shared-file' ? [true] : modes).map((followSymlinks) => ({
    name: `${kind}-${followSymlinks ? 'follow' : 'no-follow'}`,
    description:
      'should retain the registered spelling of ordinary dependencies containing parent components',
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync.native(os.tmpdir()), 'rspack-parent-components-'),
      );
      fs.mkdirSync(path.join(root, 'sub'));
      fs.mkdirSync(path.join(root, 'context'));
      const directory = kind === 'directory';
      const shared = kind === 'shared-file';
      const file = path.join(root, directory ? 'context/index.js' : 'index.js');
      const registered = `${root}${path.sep}sub${path.sep}..${path.sep}${directory ? 'context' : 'index.js'}`;
      if (kind !== 'missing') {
        fs.writeFileSync(file, 'initial');
        const past = new Date(Date.now() - 60_000);
        fs.utimesSync(file, past, past);
      }
      const watcher = new NativeWatcher({ followSymlinks, aggregateTimeout: 50 });
      const changes = new Set();
      const files = shared ? [registered, file] : kind === 'file' ? [registered] : [];
      let error;
      try {
        await sleep(2100);
        watcher.watch(
          [files, []],
          [directory ? [registered] : [], []],
          [kind === 'missing' ? [registered] : [], []],
          BigInt(Date.now()),
          (err, batch) => {
            if (err) error = err;
            else for (const changed of batch.changedFiles) changes.add(changed);
          },
          () => {},
        );
        await sleep(500);
        changes.clear();
        fs.writeFileSync(file, 'changed');
        await waitFor(() => changes.has(registered));
        if (shared) await waitFor(() => changes.has(file));
        expect([...changes].sort()).toEqual(
          (shared ? [registered, file] : [registered]).sort(),
        );
        expect(error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
