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
  ["file-change", "link-replacement"].flatMap((operation) =>
    [
      "overlap",
      "parent-only",
      "explicit-only",
      "no-ignore",
      "ignored-alias",
    ].map((scenario) => ({
      name: `${native ? "native" : "watchpack"}-${operation}-${scenario}`,
      description:
        "should filter live notifications by the subscription that received the change",
      async run() {
        const root = fs.mkdtempSync(
          path.join(
            fs.realpathSync.native(os.tmpdir()),
            "rspack-live-subscriptions-",
          ),
        );
        const app = path.join(root, "app");
        const target = path.join(root, "target");
        const outside = path.join(root, "outside");
        const link = path.join(app, "link");
        const real = path.join(target, "value.js");
        const alias = path.join(link, "value.js");
        const witness = path.join(app, "witness.js");
        const replacing = operation === "link-replacement";
        for (const dir of [app, target, outside]) fs.mkdirSync(dir);
        const old = new Date(Date.now() - 60_000);
        const targets = ["a.js", "b.js"].map((name) => {
          const file = path.join(outside, name);
          fs.writeFileSync(file, name);
          fs.utimesSync(file, old, old);
          return file;
        });
        fs.writeFileSync(witness, "before");
        fs.utimesSync(witness, old, old);
        if (replacing) fs.symlinkSync(targets[0], real, "file");
        else {
          fs.writeFileSync(real, "before");
          fs.utimesSync(real, old, old);
        }
        fs.symlinkSync(target, link, "junction");
        await sleep(2200);
        const directories = scenario === "explicit-only" ? [] : [app];
        if (replacing && scenario !== "parent-only") directories.push(link);
        const files = !replacing && scenario !== "parent-only" ? [alias] : [];
        const options = {
          followSymlinks: true,
          aggregateTimeout: 50,
          ignored: (entry) =>
            scenario === "no-ignore"
              ? false
              : scenario === "ignored-alias"
                ? entry === alias
                : entry === target || entry.startsWith(`${target}${path.sep}`),
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
          if (native)
            watcher.watch(
              [files, []],
              [directories, []],
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
            watcher.watch({ files, directories, startTime });
          }
          if (directories.includes(app))
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
          if (replacing) {
            fs.unlinkSync(real);
            fs.symlinkSync(targets[1], real, "file");
          } else fs.writeFileSync(real, "live edit");
          const expected =
            scenario === "ignored-alias"
              ? replacing
                ? [link]
                : []
              : [
                  ...(directories.includes(app) ? [app] : []),
                  ...(scenario === "no-ignore"
                    ? [replacing ? link : alias]
                    : []),
                ];
          if (expected.length)
            await waitFor(() => expected.every((entry) => changes.has(entry)));
          await sleep(500);
          expect([...changes].sort()).toEqual(expected.sort());
          expect([...removals]).toEqual([]);
          if (!replacing) {
            changes.clear();
            fs.unlinkSync(real);
            const expectedContexts = expected.filter((entry) => entry === app);
            const expectedRemovals = expected.filter(
              (entry) => entry === alias,
            );
            if (expectedContexts.length || expectedRemovals.length)
              await waitFor(
                () =>
                  expectedContexts.every((entry) => changes.has(entry)) &&
                  expectedRemovals.every((entry) => removals.has(entry)),
              );
            await sleep(500);
            expect([...changes]).toEqual(expectedContexts);
            expect([...removals]).toEqual(expectedRemovals);
            changes.clear();
            removals.clear();
            fs.writeFileSync(real, "restored");
            if (expected.length)
              await waitFor(() =>
                expected.every((entry) => changes.has(entry)),
              );
            await sleep(500);
            expect([...changes].sort()).toEqual(expected.sort());
            expect([...removals]).toEqual([]);
          }
          if (scenario === "overlap" || scenario === "explicit-only") {
            changes.clear();
            fs.unlinkSync(link);
            fs.symlinkSync(target, link, "junction");
            const expectedLinkChanges = [
              replacing ? link : alias,
              ...(directories.includes(app) ? [app] : []),
            ];
            await waitFor(() =>
              expectedLinkChanges.every((entry) => changes.has(entry)),
            );
            await sleep(500);
            expect([...changes].sort()).toEqual(expectedLinkChanges.sort());
          }
          expect(failure).toBeUndefined();
        } finally {
          await watcher.close();
          fs.rmSync(root, { recursive: true, force: true });
        }
      },
    })),
  ),
);
