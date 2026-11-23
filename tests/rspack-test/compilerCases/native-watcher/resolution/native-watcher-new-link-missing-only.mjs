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
export default [false, true].flatMap((native) =>
  [false, true].map((nested) => ({
    name: `${native ? "native" : "watchpack"}-${nested ? "nested-link" : "link"}`,
    description:
      "should recover a missing file when its directory link appears without a context dependency",
    async run() {
      const root = fs.mkdtempSync(
        path.join(fs.realpathSync(os.tmpdir()), "rspack-new-link-"),
      );
      const app = path.join(root, "app");
      const target = path.join(root, "target");
      const container = nested ? path.join(app, "packages") : app;
      const link = path.join(container, "pkg");
      const file = path.join(link, "index.js");
      fs.mkdirSync(app);
      fs.mkdirSync(target);
      fs.writeFileSync(path.join(target, "index.js"), "package");
      const old = new Date("2021-01-01T00:00:00Z");
      fs.utimesSync(path.join(target, "index.js"), old, old);
      const watcher = new (native ? NativeWatcher : Watchpack)({
        followSymlinks: true,
        aggregateTimeout: 40,
      });
      const events = createEventCollector();
      try {
        watchInitial(
          watcher,
          { missing: [file], startTime: Date.now() },
          events,
        );
        await sleep(500);
        expect([...events.changes]).toEqual([]);
        if (nested) {
          const staged = path.join(root, "staged");
          fs.mkdirSync(staged);
          fs.symlinkSync(target, path.join(staged, "pkg"), "junction");
          fs.renameSync(staged, container);
        } else fs.symlinkSync(target, link, "junction");
        await waitFor(
          () => events.changes.has(file) || events.error !== undefined,
        );
        expect(events.error).toBeUndefined();
        expect([...events.changes]).toEqual([file]);
        expect([...events.removals]).toEqual([]);
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
