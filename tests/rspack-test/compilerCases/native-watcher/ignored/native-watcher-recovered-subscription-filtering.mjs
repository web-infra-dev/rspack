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
export default [false, true].flatMap((native) =>
  [false, true].map((ignored) => ({
    name: `${native ? "native" : "watchpack"}-${ignored ? "ignored" : "visible"}`,
    description:
      "should filter recovered descendants by their own subscriptions",
    async run() {
      const root = fs.mkdtempSync(
        path.join(
          fs.realpathSync.native(os.tmpdir()),
          "rspack-recovered-subscriptions-",
        ),
      );
      const app = path.join(root, "app");
      const target = path.join(root, "target");
      const staged = path.join(root, "staged");
      for (const directory of [app, target, staged]) fs.mkdirSync(directory);
      const link = path.join(app, "link");
      const alias = path.join(link, "nested", "value.js");
      const witness = path.join(app, "witness.js");
      fs.writeFileSync(witness, "before");
      fs.writeFileSync(path.join(staged, "value.js"), "restored");
      fs.symlinkSync(target, link, "junction");
      await sleep(2200);
      const options = {
        followSymlinks: true,
        aggregateTimeout: 50,
        ignored: (entry) =>
          ignored &&
          (entry === target || entry.startsWith(`${target}${path.sep}`)),
      };
      const watcher = native
        ? new NativeWatcher(options)
        : new Watchpack(options);
      const collected = createEventCollector();
      const { changes, removals } = collected;
      try {
        const startTime = Date.now();
        watchInitial(
          watcher,
          { directories: [app], missing: [alias], startTime },
          collected,
        );
        await waitFor(() => {
          const entries = native
            ? watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.map((entry) => entry.path)
            : [...watcher.getTimeInfoEntries().keys()];
          return entries.includes(witness);
        });
        await sleep(500);
        changes.clear();
        removals.clear();
        fs.renameSync(staged, path.join(target, "nested"));
        const expected = ignored ? [app] : [app, alias];
        await waitFor(() => expected.every((entry) => changes.has(entry)));
        await sleep(500);
        expect([...changes].sort()).toEqual(expected.sort());
        expect([...removals]).toEqual([]);
        expect(collected.error).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
