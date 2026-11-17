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
        const startTime = Date.now();
        if (native) {
          watcher.watch(
            [[], []],
            [[app], []],
            [[alias], []],
            BigInt(startTime),
            (error, batch) => {
              if (error) failure = error;
              else record(batch.changedFiles, batch.removedFiles);
            },
            () => {},
          );
        } else {
          watcher.on("aggregated", record);
          watcher.watch({ directories: [app], missing: [alias], startTime });
        }
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
        expect(failure).toBeUndefined();
      } finally {
        await watcher.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  })),
);
