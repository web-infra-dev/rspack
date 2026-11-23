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
export default process.platform !== "darwin"
  ? []
  : [
      {
        name: "explicit-root-ownership",
        description:
          "should route physical directory events to each explicit root without following links in its parent context",
        async run() {
          const root = fs.mkdtempSync(
            path.join(fs.realpathSync(os.tmpdir()), "rspack-directory-roots-"),
          );
          const app = path.join(root, "app");
          const target = path.join(root, "target");
          const witness = path.join(root, "witness.js");
          const aliases = ["first", "second"].map((name) =>
            path.join(app, name),
          );
          fs.mkdirSync(app);
          fs.mkdirSync(target);
          const old = new Date("2021-01-01T00:00:00.123Z");
          fs.writeFileSync(witness, "witness");
          fs.utimesSync(witness, old, old);
          for (const name of ["value.js", "ignored.js"]) {
            const file = path.join(target, name);
            fs.writeFileSync(file, "initial");
            fs.utimesSync(file, old, old);
          }
          for (const alias of aliases) {
            fs.symlinkSync(target, alias, "dir");
            fs.lutimesSync(alias, old, old);
          }
          const ignored = (file) =>
            file === path.join(target, "value.js") ||
            file === path.join(aliases[0], "ignored.js");
          const watchers = [NativeWatcher, Watchpack].map((Watcher) => {
            const watcher = new Watcher({
              followSymlinks: false,
              aggregateTimeout: 40,
              ignored,
            });
            const batches = createEventCollector();
            const events = [];
            if (watcher instanceof Watchpack) {
              watcher.on("aggregated", batches.record);
              watcher.on("change", (file) => events.push(file));
              watcher.on("remove", (file) => events.push(file));
            }
            return { watcher, batches, events };
          });
          const watch = async (state, removed = []) => {
            const startTime = Date.now();
            if (state.watcher instanceof NativeWatcher) {
              await state.watcher.watch(
                [removed.length ? [] : [witness], []],
                [removed.length ? [] : [app, ...aliases], removed],
                [[], []],
                BigInt(startTime),
                state.batches.callback,
                (event) => state.events.push(event.path),
              );
            } else {
              state.watcher.watch({
                files: [witness],
                directories: [
                  app,
                  ...aliases.filter((alias) => !removed.includes(alias)),
                ],
                startTime,
              });
            }
          };
          const info = (state) => {
            if (state.watcher instanceof NativeWatcher) {
              const entries = state.watcher.collectTimeInfoEntries();
              return {
                files: new Map(
                  entries.fileTimestamps.entries.map((entry) => [
                    entry.path,
                    entry,
                  ]),
                ),
                dirs: new Map(
                  entries.directoryTimestamps.entries.map((entry) => [
                    entry.path,
                    entry,
                  ]),
                ),
              };
            }
            const files = new Map(),
              dirs = new Map();
            state.watcher.collectTimeInfoEntries(files, dirs);
            return { files, dirs };
          };
          const reset = () => {
            for (const { events, batches } of watchers) {
              events.length = 0;
              batches.changes.clear();
              batches.removals.clear();
            }
          };
          try {
            for (const state of watchers) await watch(state);
            await waitFor(() =>
              watchers.every((state) =>
                aliases.every((alias) =>
                  info(state).files.has(path.join(alias, "value.js")),
                ),
              ),
            );
            await sleep(300);
            const parentTimes = watchers.map(
              (state) => info(state).dirs.get(app).safeTime,
            );
            reset();
            for (const [name, expected] of [
              ["value.js", aliases],
              ["ignored.js", aliases.slice(1)],
            ]) {
              fs.writeFileSync(path.join(target, name), "changed");
              await waitFor(() =>
                watchers.every(({ batches }) =>
                  expected.every((alias) => batches.changes.has(alias)),
                ),
              );
              await sleep(150);
              for (const [index, state] of watchers.entries()) {
                expect([...state.batches.changes].sort()).toEqual(
                  [...expected].sort(),
                );
                expect([...new Set(state.events)].sort()).toEqual(
                  expected.map((alias) => path.join(alias, name)).sort(),
                );
                expect(info(state).dirs.get(app).safeTime).toBe(
                  parentTimes[index],
                );
              }
              reset();
            }
            for (const state of watchers) await watch(state, [aliases[0]]);
            await sleep(200);
            reset();
            fs.symlinkSync(
              path.join(root, "missing.js"),
              path.join(target, "dangling.js"),
            );
            await waitFor(() =>
              watchers.every(({ batches }) => batches.changes.has(aliases[1])),
            );
            await sleep(150);
            for (const [index, state] of watchers.entries()) {
              expect([...state.batches.changes]).toEqual([aliases[1]]);
              expect([...new Set(state.events)]).toEqual([
                path.join(aliases[1], "dangling.js"),
              ]);
              expect(
                info(state).files.has(path.join(aliases[0], "value.js")),
              ).toBe(false);
              expect(
                info(state).files.has(path.join(aliases[1], "dangling.js")),
              ).toBe(true);
              expect(info(state).dirs.get(app).safeTime).toBe(
                parentTimes[index],
              );
              expect(state.batches.error).toBeUndefined();
            }
            reset();
            fs.rmSync(target, { recursive: true });
            // The physical parent watch observes the deletion; Watchpack's
            // alias watcher can stay attached to the removed directory.
            await waitFor(
              () =>
                watchers[0].batches.changes.has(aliases[1]) ||
                watchers[0].batches.removals.has(aliases[1]),
            );
            for (const state of watchers) await watch(state, aliases);
            await sleep(200);
            for (const [index, state] of watchers.entries()) {
              expect(state.batches.changes.has(app)).toBe(false);
              expect(info(state).dirs.get(app).safeTime).toBe(
                parentTimes[index],
              );
              for (const alias of aliases) {
                expect(info(state).files.has(alias)).toBe(true);
                expect(info(state).dirs.has(alias)).toBe(false);
                expect(
                  info(state).files.has(path.join(alias, "value.js")),
                ).toBe(false);
              }
            }
          } finally {
            for (const { watcher } of watchers) await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      },
      {
        name: "registered-parent",
        description:
          "should retain a registered parent root when its only registered child is a directory link",
        async run() {
          const root = fs.mkdtempSync(
            path.join(fs.realpathSync(os.tmpdir()), "rspack-root-parent-"),
          );
          const app = path.join(root, "app");
          const target = path.join(root, "target");
          const link = path.join(app, "link");
          const file = path.join(app, "value.js");
          fs.mkdirSync(app);
          fs.mkdirSync(target);
          fs.symlinkSync(target, link, "dir");
          fs.writeFileSync(file, "initial");
          const old = new Date("2021-01-01T00:00:00.123Z");
          fs.utimesSync(file, old, old);
          fs.lutimesSync(link, old, old);
          const states = [NativeWatcher, Watchpack].map((Watcher) => ({
            watcher: new Watcher({
              followSymlinks: false,
              aggregateTimeout: 40,
            }),
            events: createEventCollector(),
          }));
          try {
            for (const { watcher, events } of states)
              watchInitial(
                watcher,
                { directories: [app, link], startTime: Date.now() },
                events,
              );
            await waitFor(() =>
              states.every(({ watcher }) =>
                watcher instanceof NativeWatcher
                  ? watcher
                      .collectTimeInfoEntries()
                      .fileTimestamps.entries.some(
                        (entry) => entry.path === file,
                      )
                  : watcher.getTimeInfoEntries().has(file),
              ),
            );
            await sleep(200);
            for (const { events } of states) events.changes.clear();
            fs.writeFileSync(file, "changed");
            await waitFor(() =>
              states.every(({ events }) => events.changes.has(app)),
            );
            for (const { events } of states)
              expect([...events.changes]).toEqual([app]);
          } finally {
            for (const { watcher } of states) await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      },
    ];
