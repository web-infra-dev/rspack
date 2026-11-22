import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import {
  NativeWatcher,
  createEventCollector,
  waitFor,
  watchInitial,
} from "../_helpers/watcher.mjs";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    "should discard the old target when a directory link changes during the initial scan",
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), "rspack-scan-retarget-"),
    );
    const app = path.join(root, "app");
    const link = path.join(app, "link");
    const targets = ["a", "b"].map((name) => path.join(root, name));
    fs.mkdirSync(app);
    for (const [index, target] of targets.entries()) {
      fs.mkdirSync(target);
      fs.writeFileSync(path.join(target, `${index}.js`), "initial");
    }
    fs.symlinkSync(targets[0], link, "junction");
    let retargeted = false;
    const watcher = new NativeWatcher({
      followSymlinks: true,
      aggregateTimeout: 40,
      ignored(entry) {
        if (!retargeted && entry === path.join(targets[0], "0.js")) {
          retargeted = true;
          const replacement = path.join(app, "replacement");
          fs.symlinkSync(targets[1], replacement, "junction");
          if (process.platform === "win32") fs.unlinkSync(link);
          fs.renameSync(replacement, link);
          fs.rmSync(targets[0], { recursive: true });
        }
        return false;
      },
    });
    const events = createEventCollector();
    try {
      watchInitial(
        watcher,
        { directories: [link], startTime: Date.now() },
        events,
      );
      await waitFor(() => retargeted || events.error !== undefined);
      await waitFor(
        () => events.changes.has(link) || events.error !== undefined,
      );
      expect(events.error).toBeUndefined();
      const entries = () =>
        watcher
          .collectTimeInfoEntries()
          .fileTimestamps.entries.map((entry) => entry.path);
      await waitFor(() => entries().includes(path.join(link, "1.js")));
      expect(entries()).not.toContain(path.join(link, "0.js"));
      await sleep(300);
      events.changes.clear();
      fs.writeFileSync(path.join(targets[1], "1.js"), "after retarget");
      await waitFor(
        () => events.changes.has(link) || events.error !== undefined,
      );
      expect(events.error).toBeUndefined();
      expect(fs.readFileSync(path.join(link, "1.js"), "utf8")).toBe(
        "after retarget",
      );
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
};
