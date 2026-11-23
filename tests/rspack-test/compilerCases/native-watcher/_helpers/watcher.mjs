import { createRequire } from 'node:module';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
export const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);
export const Watchpack = require(
  path.join(
    path.dirname(require.resolve('@rspack/core/package.json')),
    'compiled/watchpack',
  ),
);

export async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

export function watchInitial(watcher, dependencies, { callback, record }) {
  if (watcher instanceof NativeWatcher) {
    const { files = [], directories = [], missing = [], startTime } = dependencies;
    watcher.watch(
      [files, []],
      [directories, []],
      [missing, []],
      BigInt(startTime),
      callback,
      () => {},
    );
  } else {
    watcher.on('aggregated', record);
    watcher.watch(dependencies);
  }
}

export function createEventCollector() {
  const changes = new Set();
  const removals = new Set();
  let failure;
  const record = (changed, removed) => {
    for (const file of changed) changes.add(file);
    for (const file of removed) removals.add(file);
  };
  return {
    changes,
    removals,
    record,
    callback(error, batch) {
      if (error) failure = error;
      else record(batch.changedFiles, batch.removedFiles);
    },
    get error() {
      return failure;
    },
  };
}
