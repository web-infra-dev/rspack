import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  NativeWatcher,
  Watchpack,
  waitFor,
  createEventCollector,
  watchInitial,
} from "../_helpers/watcher.mjs";

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
    const collected = createEventCollector();
    const { changes, removals } = collected;
    try {
      watchInitial(watcher, { directories: [app], startTime }, collected);
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
      expect(collected.error).toBeUndefined();
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
