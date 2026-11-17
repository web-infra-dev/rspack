import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  NativeWatcher,
  Watchpack,
  createEventCollector,
  waitFor,
} from "../_helpers/watcher.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  { name: "direct", followSymlinks: false },
  { name: "direct-follow", followSymlinks: true },
  { name: "link", followSymlinks: true, link: true },
].map((variant) => ({
  name: variant.name,
  description:
    "should keep parent watch notifications out of concrete events and ignored changes",
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), "rspack-parent-notifications-"),
    );
    const target = path.join(root, "target");
    const directory = variant.link ? path.join(root, "link") : target;
    fs.mkdirSync(target);
    if (variant.link) fs.symlinkSync(target, directory, "junction");
    const oldTime = new Date("2021-01-01T00:00:00Z");
    for (const name of ["child.js", "ignored.js"]) {
      const file = path.join(target, name);
      fs.writeFileSync(file, "initial");
      fs.utimesSync(file, oldTime, oldTime);
    }
    fs.utimesSync(target, oldTime, oldTime);
    const options = {
      aggregateTimeout: 40,
      followSymlinks: variant.followSymlinks,
      ignored: (file) => path.basename(file) === "ignored.js",
    };
    const native = new NativeWatcher(options);
    const watchpack = new Watchpack(options);
    const nativeEvents = [];
    const watchpackEvents = [];
    const nativeBatches = createEventCollector();
    const watchpackBatches = createEventCollector();
    watchpack.on("change", (file) =>
      watchpackEvents.push({ kind: "change", path: file }),
    );
    watchpack.on("remove", (file) =>
      watchpackEvents.push({ kind: "remove", path: file }),
    );
    watchpack.on("aggregated", watchpackBatches.record);
    const startTime = Date.now();
    try {
      await native.watch(
        [[], []],
        [[directory], []],
        [[], []],
        BigInt(startTime),
        nativeBatches.callback,
        ({ kind, path }) => nativeEvents.push({ kind, path }),
      );
      watchpack.watch({ directories: [directory], startTime });
      const child = path.join(directory, "child.js");
      const realChild = path.join(target, "child.js");
      await waitFor(
        () =>
          native.collectTimeInfoEntries().fileTimestamps.entries.some(
            (entry) => entry.path === child || entry.path === realChild,
          ) &&
          (watchpack.getTimeInfoEntries().has(child) ||
            watchpack.getTimeInfoEntries().has(realChild)),
      );
      await sleep(200);
      nativeEvents.length = watchpackEvents.length = 0;
      nativeBatches.changes.clear();
      watchpackBatches.changes.clear();

      fs.unlinkSync(path.join(target, "ignored.js"));
      fs.writeFileSync(path.join(root, "unrelated.js"), "unrelated");
      await sleep(300);
      for (const events of [nativeEvents, watchpackEvents]) {
        expect(events).toEqual([]);
      }
      for (const batch of [nativeBatches, watchpackBatches]) {
        expect([...batch.changes]).toEqual([]);
        expect([...batch.removals]).toEqual([]);
      }

      fs.unlinkSync(realChild);
      await waitFor(
        () =>
          nativeBatches.changes.has(directory) &&
          watchpackBatches.changes.has(directory),
      );
      await sleep(200);
      const normalize = (events) =>
        [...new Set(events.map((event) => JSON.stringify(event)))].sort();
      expect(normalize(nativeEvents)).toEqual(normalize(watchpackEvents));
      expect(nativeEvents.every((event) => event.path !== directory)).toBe(true);
      expect(nativeBatches.error).toBeUndefined();
    } finally {
      watchpack.close();
      await native.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
