import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  NativeWatcher,
  createEventCollector,
  waitFor,
} from "../_helpers/watcher.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default ["link-file", "direct-file", "missing", "remove"].map(
  (scenario) => ({
    name: scenario,
    description:
      "should backfill file dependencies after their directory watch is reattached without a context dependency",
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), "rspack-file-reattach-"),
      );
      const target = path.join(root, "target");
      const replacement = path.join(root, "replacement");
      const oldTime = new Date("2021-01-01T00:00:00Z");
      for (const directory of [target, replacement]) {
        fs.mkdirSync(directory);
        for (const name of ["value.js", "unchanged.js", "ignored.js"]) {
          const file = path.join(directory, name);
          fs.writeFileSync(file, "initial");
          fs.utimesSync(file, oldTime, oldTime);
        }
      }
      const entry =
        scenario === "direct-file" ? target : path.join(root, "link");
      if (entry !== target) fs.symlinkSync(target, entry, "junction");
      const file = path.join(entry, "value.js");
      const missing = path.join(entry, "missing.js");
      const unchanged = path.join(entry, "unchanged.js");
      const ignored = path.join(entry, "ignored.js");
      const watcher = new NativeWatcher({
        followSymlinks: true,
        aggregateTimeout: 40,
        ignored: (file) => file === path.join(target, "ignored.js"),
      });
      const events = createEventCollector();
      const watch = (initial, startTime) =>
        watcher.watch(
          [initial ? [file, unchanged, ignored] : [], []],
          [[], []],
          [initial && scenario === "missing" ? [missing] : [], []],
          BigInt(startTime),
          events.callback,
          () => {},
        );
      try {
        await watch(true, Date.now());
        await sleep(500);
        events.changes.clear();
        events.removals.clear();
        fs.renameSync(target, path.join(root, "previous"));
        fs.renameSync(replacement, target);
        fs.writeFileSync(file, "read by the build");
        await waitFor(() => events.changes.has(file));
        await sleep(300);
        watcher.pause();
        const startTime = Date.now();
        expect(fs.readFileSync(file, "utf8")).toBe("read by the build");
        expect(fs.existsSync(missing)).toBe(false);
        await sleep(30);
        if (scenario === "remove") fs.unlinkSync(file);
        else if (scenario === "missing")
          fs.writeFileSync(missing, "created after read");
        else fs.writeFileSync(file, "changed after read");
        fs.writeFileSync(ignored, "ignored change after read");
        await sleep(300);
        events.changes.clear();
        events.removals.clear();
        await watch(false, startTime);
        const expected =
          scenario === "remove" ? events.removals : events.changes;
        await waitFor(() =>
          expected.has(scenario === "missing" ? missing : file),
        );
        expect(events.changes.has(ignored)).toBe(false);
        expect(events.changes.has(unchanged)).toBe(false);
        events.changes.clear();
        fs.writeFileSync(file, "after rewatch");
        await waitFor(() => events.changes.has(file));
        expect(events.error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  }),
);
