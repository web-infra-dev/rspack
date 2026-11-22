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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default process.platform !== "darwin"
  ? []
  : [false, true].map((linked) => ({
      name: linked ? "linked" : "direct",
      description:
        "should report removal of an empty symlinked context by its registered path when following is disabled",
      async run() {
        const root = fs.mkdtempSync(
          path.join(fs.realpathSync(os.tmpdir()), "rspack-empty-root-"),
        );
        const app = path.join(root, "app");
        const target = path.join(root, "target");
        const link = linked ? path.join(app, "link") : target;
        fs.mkdirSync(app);
        fs.mkdirSync(target);
        if (linked) fs.symlinkSync(target, link, "dir");
        const watcher = new NativeWatcher({
          followSymlinks: false,
          aggregateTimeout: 40,
        });
        const events = createEventCollector();
        try {
          watchInitial(
            watcher,
            { directories: [link], startTime: Date.now() },
            events,
          );
          await sleep(600);
          events.changes.clear();
          events.removals.clear();
          fs.rmdirSync(target);
          await waitFor(
            () => events.removals.has(link) || events.error !== undefined,
          );
          expect(events.error).toBeUndefined();
          expect([...events.removals]).toEqual([link]);
          expect([...events.changes]).toEqual([]);
        } finally {
          await watcher.close();
          fs.rmSync(root, { recursive: true, force: true });
        }
      },
    }));
