import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
  "@rspack/binding",
);

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

function fixture() {
  return fs.mkdtempSync(
    path.join(fs.realpathSync(os.tmpdir()), "rspack-watch-replay-"),
  );
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [
  {
    name: "context-to-file",
    description:
      "replays a real file change to the new file subscription after removing its context",
    async run() {
      const root = fixture();
      const file = path.join(root, "file.js");
      fs.writeFileSync(file, "before");
      const before = new Date(Date.now() - 120000);
      fs.utimesSync(file, before, before);
      const watcher = new NativeWatcher({ aggregateTimeout: 40 });
      const start = BigInt(Date.now() + 60000);
      const none = [[], []];
      const marker = path.join(root, "ready");
      let sawMarker = false;
      let initialBatchDelivered = false;
      let oldFileEvents = 0;
      const raw = [];
      const batches = [];
      const timestamp = () =>
        watcher
          .collectTimeInfoEntries()
          .fileTimestamps.entries.find((row) => row.path === file)?.timestamp;
      try {
        watcher.watch(
          none,
          [[root], []],
          none,
          start,
          (error) => {
            expect(error).toBe(null);
            if (sawMarker) initialBatchDelivered = true;
          },
          (event) => {
            if (event.path === marker) sawMarker = true;
            if (event.path === file) oldFileEvents++;
          },
        );
        await waitFor(() => timestamp() === before.getTime());
        // Drain callbacks for fixture setup before testing the pause boundary.
        watcher.triggerEvent("remove", marker);
        await waitFor(() => initialBatchDelivered);
        const beforePause = oldFileEvents;
        watcher.pause();
        fs.writeFileSync(file, "after");
        const after = new Date(before.getTime() + 60000);
        fs.utimesSync(file, after, after);
        await waitFor(
          () =>
            timestamp() === after.getTime() &&
            watcher.peekAggregated().changedFiles.includes(root),
        );
        expect(oldFileEvents).toBe(beforePause);
        watcher.watch(
          [[file], []],
          [[], [root]],
          none,
          start,
          (error, batch) => {
            expect(error).toBe(null);
            batches.push(batch);
          },
          (event) => raw.push(event.path),
        );
        await waitFor(() => batches.length > 0);
        expect(raw).toContain(file);
        expect(new Set(batches.flatMap((batch) => batch.changedFiles))).toEqual(
          new Set([file]),
        );
        expect(batches.flatMap((batch) => batch.removedFiles)).toEqual([]);
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  },
  {
    name: "close-from-ignored",
    description:
      "can close a directory scan while its JavaScript ignored predicate is running",
    async run() {
      const root = fixture();
      const file = path.join(root, "file.js");
      fs.writeFileSync(file, "");
      let closing;
      const watcher = new NativeWatcher({
        ignored(name) {
          if (name === file && !closing) {
            closing = watcher.close();
            Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 150);
          }
          return false;
        },
      });
      try {
        watcher.watch(
          [[], []],
          [[root], []],
          [[], []],
          BigInt(Date.now() + 60000),
          () => {},
          () => {},
        );
        await waitFor(() => Boolean(closing));
        await closing;
      } finally {
        await (closing ?? watcher.close());
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  },
];
