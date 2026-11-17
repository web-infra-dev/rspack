import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  NativeWatcher,
  Watchpack,
  createEventCollector,
  waitFor,
  watchInitial,
} from "../_helpers/watcher.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    name: "file-to-directory",
    description:
      "should replace a file record when a delayed event observes a directory in its place",
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), "rspack-entry-type-"),
      );
      const child = path.join(root, "child");
      const file = path.join(child, "value.js");
      fs.writeFileSync(child, "initial");
      const old = new Date("2021-01-01T00:00:00.123Z");
      fs.utimesSync(child, old, old);
      const states = [NativeWatcher, Watchpack].map((Watcher) => ({
        watcher: new Watcher({
          followSymlinks: false,
          aggregateTimeout: 40,
          ignored: () => false,
        }),
        events: createEventCollector(),
      }));
      const files = (watcher) =>
        watcher instanceof NativeWatcher
          ? new Set(
              watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.map((entry) => entry.path),
            )
          : new Set(watcher.getTimeInfoEntries().keys());
      try {
        for (const { watcher, events } of states)
          watchInitial(
            watcher,
            { directories: [root], startTime: Date.now() },
            events,
          );
        await waitFor(() =>
          states.every(({ watcher }) => files(watcher).has(child)),
        );
        await sleep(200);
        for (const { events } of states) events.changes.clear();
        fs.unlinkSync(child);
        fs.mkdirSync(child);
        fs.writeFileSync(file, "in replacement directory");
        await waitFor(() =>
          states.every(
            ({ watcher, events }) =>
              events.changes.has(root) && files(watcher).has(file),
          ),
        );
        await sleep(200);
        for (const { watcher, events } of states) {
          const startTime = Date.now();
          if (watcher instanceof NativeWatcher) {
            await watcher.watch(
              [[file], []],
              [[], []],
              [[], []],
              BigInt(startTime),
              events.callback,
              () => {},
            );
          } else {
            watcher.watch({ files: [file], directories: [root], startTime });
          }
        }
        for (const { events } of states) events.changes.clear();
        fs.writeFileSync(file, "later edit");
        await waitFor(() =>
          states.every(({ events }) => events.changes.has(root)),
        );
        for (const { events } of states) expect(events.error).toBeUndefined();
      } finally {
        for (const { watcher } of states) await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  },
];
