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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  'new-directory',
  'dangling-directory',
  'no-follow',
  // These cases exercise POSIX traversal of parent components through links.
  ...(process.platform === 'win32' ? [] : ['parent-component', 'missing-parent-component']),
].map((scenario) => ({
  name: scenario,
  description: `should discover symbolic link targets for ${scenario}`,
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), 'rspack-link-discovery-'),
    );
    const source = path.join(root, 'app');
    const target = path.join(root, 'outside', 'package');
    const realFile = path.join(target, 'value.js');
    const link = path.join(source, 'link');
    const parentComponent = scenario.includes('parent-component');
    const missing = scenario === 'missing-parent-component';
    const followSymlinks = scenario !== 'no-follow';
    fs.mkdirSync(source);
    fs.mkdirSync(path.join(target, 'subdir'), { recursive: true });
    if (!missing) {
      fs.writeFileSync(realFile, 'initial');
      const past = new Date(Date.now() - 60_000);
      fs.utimesSync(realFile, past, past);
    }
    const bridge = path.join(source, 'bridge');
    if (parentComponent) {
      fs.symlinkSync(path.join(target, 'subdir'), bridge, 'dir');
      // Do not use path.join: it would erase the component under test.
      fs.symlinkSync('bridge/../value.js', link, 'file');
      if (!missing) expect(fs.realpathSync.native(link)).toBe(realFile);
    } else if (scenario === 'dangling-directory') {
      fs.rmSync(target, { recursive: true });
      fs.symlinkSync(target, link, 'junction');
    }
    await sleep(2100);
    const watcher = new NativeWatcher({ followSymlinks, aggregateTimeout: 50 });
    const changes = new Set();
    const events = new Set();
    let error;
    const files = parentComponent && !missing ? [link] : [];
    const directories = parentComponent ? [] : [source];
    const missingFiles = missing ? [link] : [];
    const watch = (initial, remove = false) => {
      watcher.watch(
        [initial ? files : [], remove ? files : []],
        [initial ? directories : [], remove ? directories : []],
        [initial ? missingFiles : [], remove ? missingFiles : []],
        BigInt(Date.now()),
        (err, batch) => {
          if (err) error = err;
          else for (const file of batch.changedFiles) changes.add(file);
        },
        (event) => events.add(event.path),
      );
    };
    watch(true);
    try {
      await sleep(500);
      changes.clear();
      events.clear();
      if (!parentComponent) {
        if (scenario === 'dangling-directory') {
          fs.mkdirSync(target, { recursive: true });
          fs.writeFileSync(realFile, 'restored');
        } else {
          fs.symlinkSync(target, link, 'junction');
        }
        await waitFor(() => changes.has(source));
        watch(false);
        await sleep(500);
        changes.clear();
        events.clear();
      }

      fs.writeFileSync(realFile, 'changed');
      const dependency = parentComponent ? link : source;
      const aliasFile = parentComponent ? link : path.join(link, 'value.js');
      if (followSymlinks) {
        await waitFor(() => changes.has(dependency));
        expect(events.has(aliasFile)).toBe(true);
      } else {
        await sleep(700);
        expect([...changes]).toEqual([]);
        expect([...events]).toEqual([]);
      }

      if (parentComponent) {
        const next = path.join(root, 'next');
        fs.mkdirSync(path.join(next, 'subdir'), { recursive: true });
        fs.writeFileSync(path.join(next, 'value.js'), 'replacement');
        changes.clear();
        fs.unlinkSync(bridge);
        fs.symlinkSync(path.join(next, 'subdir'), bridge, 'dir');
        await waitFor(() => changes.has(link));
        watch(false);
        await sleep(500);
        changes.clear();
        fs.writeFileSync(path.join(next, 'value.js'), 'after rewatch');
        await waitFor(() => changes.has(link));
        changes.clear();
        fs.writeFileSync(realFile, 'old target');
        await sleep(700);
        expect([...changes]).toEqual([]);
      }

      watch(false, true);
      await sleep(500);
      changes.clear();
      events.clear();
      fs.writeFileSync(realFile, 'after unregister');
      await sleep(700);
      expect([...events]).toEqual([]);
      expect(error).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
