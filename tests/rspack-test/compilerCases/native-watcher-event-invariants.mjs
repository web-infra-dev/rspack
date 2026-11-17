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
const old = new Date("2021-01-01T00:00:00Z");

function writeOld(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
  fs.utimesSync(file, old, old);
}

async function withWatcher(native, options, setup, run) {
  const root = fs.mkdtempSync(
    path.join(fs.realpathSync.native(os.tmpdir()), "rspack-event-invariants-"),
  );
  const app = path.join(root, "app");
  const witness = path.join(app, "witness.js");
  writeOld(witness, "before");
  const fixture = setup({ root, app, witness });
  fs.utimesSync(app, old, old);
  const config = { aggregateTimeout: 40, ...options(fixture) };
  const watcher = native ? new NativeWatcher(config) : new Watchpack(config);
  const changes = new Set();
  const removals = new Set();
  const immediate = new Set();
  let failure;
  const collect = (changed, removed) => {
    for (const entry of changed) changes.add(entry);
    for (const entry of removed) removals.add(entry);
  };
  const clear = () => {
    changes.clear();
    removals.clear();
    immediate.clear();
  };
  const waitFor = async (predicate) => {
    const deadline = Date.now() + 5000;
    while (!predicate() && !failure && Date.now() < deadline) await sleep(10);
    expect(failure).toBeUndefined();
    expect(predicate()).toBe(true);
  };
  const timestamps = () => {
    if (!native) return watcher.getTimeInfoEntries();
    const table = watcher.collectTimeInfoEntries().fileTimestamps;
    return new Map([
      ...table.entries.map((entry) => [entry.path, entry]),
      ...table.missing.map((entry) => [entry, null]),
    ]);
  };
  const barrier = async (value) => {
    clear();
    fs.writeFileSync(witness, value);
    await waitFor(() =>
      changes.has(fixture.directories?.includes(app) ? app : witness),
    );
    await sleep(200);
    clear();
  };
  try {
    const startTime = Date.now();
    const { files = [], directories = [], missing = [] } = fixture;
    if (native) {
      watcher.watch(
        [files, []],
        [directories, []],
        [missing, []],
        BigInt(startTime),
        (error, batch) => {
          if (error) failure = error;
          else collect(batch.changedFiles, batch.removedFiles);
        },
        (event) => immediate.add(event.path),
      );
    } else {
      watcher.on("aggregated", collect);
      watcher.on("change", (entry) => immediate.add(entry));
      watcher.watch({ files, directories, missing, startTime });
    }
    await waitFor(() => timestamps().has(witness));
    await sleep(250);
    await barrier("ready");
    await run({
      ...fixture,
      root,
      app,
      witness,
      changes,
      removals,
      immediate,
      timestamps,
      waitFor,
      barrier,
    });
    await barrier("finished");
    expect(failure).toBeUndefined();
  } finally {
    await watcher.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  ...[false, true].flatMap((native) =>
    [false, true].map((followSymlinks) => ({
      name: `${native ? "native" : "watchpack"}-ignored-ancestor-${followSymlinks}`,
      description:
        "should not notify a parent context for an ignored ancestor of a missing dependency",
      run: () =>
        withWatcher(
          native,
          ({ dist }) => ({
            followSymlinks,
            ignored: (entry) => entry === dist,
          }),
          ({ app }) => {
            const dist = path.join(app, "dist");
            const missingFile = path.join(dist, "entry.js");
            return {
              dist,
              missingFile,
              directories: [app],
              missing: [missingFile],
            };
          },
          async ({
            dist,
            missingFile,
            changes,
            removals,
            immediate,
            waitFor,
          }) => {
            fs.mkdirSync(dist);
            await sleep(800);
            expect(fs.existsSync(missingFile)).toBe(false);
            expect([...changes]).toEqual([]);
            expect([...removals]).toEqual([]);
            expect([...immediate]).toEqual([]);
            if (!native) return;
            fs.writeFileSync(missingFile, "created");
            await waitFor(() => changes.has(missingFile));
            await sleep(200);
            expect([...changes]).toEqual([missingFile]);
            expect([...immediate]).toEqual([missingFile]);
          },
        ),
    })),
  ),
  ...[false, true].flatMap((native) =>
    [false, true].map((registerLink) => ({
      name: `${native ? "native" : "watchpack"}-directory-link-file-dependency-${registerLink}`,
      description:
        "should preserve a context subscription when its directory link is also a file dependency",
      run: () =>
        withWatcher(
          native,
          () => ({ followSymlinks: true }),
          ({ root, app }) => {
            const target = path.join(root, "target", "value.js");
            const link = path.join(app, "link");
            writeOld(target, "before");
            fs.symlinkSync(path.dirname(target), link, "junction");
            if (process.platform !== "win32") fs.lutimesSync(link, old, old);
            return {
              target,
              alias: path.join(link, "value.js"),
              files: registerLink ? [link] : [],
              directories: [app],
            };
          },
          async ({
            app,
            target,
            alias,
            changes,
            immediate,
            timestamps,
            waitFor,
          }) => {
            await waitFor(() => timestamps().has(alias));
            fs.writeFileSync(target, "changed target");
            await waitFor(() => changes.has(app));
            expect(immediate.has(alias)).toBe(true);
            expect(timestamps().get(alias).timestamp).toBeGreaterThan(
              old.getTime(),
            );
          },
        ),
    })),
  ),
  ...[false, true].map((followSymlinks) => ({
    name: `native-observed-ancestor-removal-${followSymlinks}`,
    description:
      "should discard descendant timestamps after an observed ancestor removal before recovering equal mtimes",
    run: () =>
      withWatcher(
        true,
        () => ({ followSymlinks }),
        ({ root, app, witness }) => {
          const tree = path.join(app, "assets");
          const next = path.join(root, "next");
          const file = path.join(tree, "nested", "value.js");
          writeOld(file, "before");
          writeOld(path.join(next, "nested", "value.js"), "different content");
          return { tree, next, file, files: [file, witness] };
        },
        async ({
          root,
          tree,
          next,
          file,
          changes,
          timestamps,
          waitFor,
          barrier,
        }) => {
          await waitFor(
            () => timestamps().get(file)?.timestamp === old.getTime(),
          );
          fs.renameSync(tree, path.join(root, "away"));
          await barrier("moved out");
          expect(fs.existsSync(file)).toBe(false);
          await waitFor(() => timestamps().get(file) === null);
          fs.renameSync(next, tree);
          await waitFor(() => changes.has(file));
          expect(fs.readFileSync(file, "utf8")).toBe("different content");
          expect(fs.statSync(file).mtimeMs).toBe(old.getTime());
          expect(timestamps().get(file)?.timestamp).toBe(old.getTime());
        },
      ),
  })),
  ...(process.platform === "win32"
    ? []
    : [false, true].map((preserveMtime) => ({
        name: `native-retarget-link-preserved-mtime-${preserveMtime}`,
        description:
          "should detect a changed link target independently of the link modification time",
        run: () =>
          withWatcher(
            true,
            () => ({ followSymlinks: true }),
            ({ root, app, witness }) => {
              const first = path.join(root, "targets", "a.js");
              const second = path.join(root, "targets", "b.js");
              const link = path.join(app, "link.js");
              const next = path.join(root, "next-link");
              writeOld(first, "A");
              writeOld(second, "B");
              const later = new Date("2021-02-01T00:00:00Z");
              fs.utimesSync(second, later, later);
              fs.symlinkSync(first, link);
              fs.lutimesSync(link, old, old);
              fs.symlinkSync(second, next);
              if (preserveMtime) fs.lutimesSync(next, old, old);
              return { link, next, files: [link, witness] };
            },
            async ({ link, next, changes, immediate, waitFor }) => {
              expect(fs.readFileSync(link, "utf8")).toBe("A");
              fs.renameSync(next, link);
              expect(fs.lstatSync(link).mtimeMs === old.getTime()).toBe(
                preserveMtime,
              );
              expect(fs.readFileSync(link, "utf8")).toBe("B");
              await waitFor(() => changes.has(link));
              expect(immediate.has(link)).toBe(true);
            },
          ),
      }))),
];
