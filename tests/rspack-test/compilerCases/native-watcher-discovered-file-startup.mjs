import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
  "@rspack/binding",
);
const Watchpack = require(
  path.join(
    path.dirname(require.resolve("@rspack/core/package.json")),
    "compiled/watchpack",
  ),
);

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [false, true].map((native) => ({
  name: native ? "native" : "watchpack",
  description:
    "should start discovered file target subscriptions when watching begins",
  async run() {
    const root = fs.mkdtempSync(
      path.join(
        fs.realpathSync.native(os.tmpdir()),
        "rspack-discovered-file-startup-",
      ),
    );
    const app = path.join(root, "app");
    const outside = path.join(root, "outside");
    fs.mkdirSync(app);
    fs.mkdirSync(outside);
    const target = path.join(outside, "value.js");
    const link = path.join(app, "link.js");
    fs.writeFileSync(target, "before");
    fs.symlinkSync(target, link, "file");
    await sleep(2200);
    const startTime = Date.now();
    await sleep(50);
    fs.writeFileSync(target, "changed during build");
    await sleep(2200);
    const options = { followSymlinks: true, aggregateTimeout: 50 };
    const watcher = native
      ? new NativeWatcher(options)
      : new Watchpack(options);
    const changes = new Set();
    const removals = new Set();
    let failure;
    const record = (changed, removed) => {
      for (const entry of changed) changes.add(entry);
      for (const entry of removed) removals.add(entry);
    };
    const waitFor = async (predicate) => {
      const deadline = Date.now() + 5000;
      while (!predicate() && Date.now() < deadline) await sleep(10);
      expect(predicate()).toBe(true);
    };
    try {
      if (native)
        watcher.watch(
          [[], []],
          [[app], []],
          [[], []],
          BigInt(startTime),
          (error, batch) => {
            if (error) failure = error;
            else record(batch.changedFiles, batch.removedFiles);
          },
          () => {},
        );
      else {
        watcher.on("aggregated", record);
        watcher.watch({ directories: [app], startTime });
      }
      await waitFor(() => {
        const entries = native
          ? watcher
              .collectTimeInfoEntries()
              .fileTimestamps.entries.map((entry) => entry.path)
          : [...watcher.getTimeInfoEntries().keys()];
        return entries.includes(link);
      });
      await sleep(700);
      expect([...changes]).toEqual([]);
      expect([...removals]).toEqual([]);
      fs.writeFileSync(target, "live edit");
      await waitFor(() => changes.has(app));
      expect([...changes]).toEqual([app]);
      expect(failure).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
