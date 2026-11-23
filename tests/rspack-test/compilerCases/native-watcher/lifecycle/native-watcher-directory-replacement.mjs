import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { NativeWatcher } from "../_helpers/watcher.mjs";

const oldTime = new Date("2021-01-01T00:00:00Z");

function writeOld(file) {
  fs.writeFileSync(file, path.basename(file));
  fs.utimesSync(file, oldTime, oldTime);
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  { name: "direct", followSymlinks: false },
  { name: "direct-follow", followSymlinks: true },
  { name: "link", followSymlinks: true, link: true },
  { name: "link-rewatch-gap", followSymlinks: true, link: true, gap: true },
  {
    name: "link-immediate-rewatch",
    followSymlinks: true,
    link: true,
    rewatch: true,
  },
  { name: "nested-link", followSymlinks: true, link: true, nested: true },
  { name: "link-with-files", followSymlinks: true, link: true, files: true },
].map((variant) => ({
  name: variant.name,
  description:
    "should reconcile a replaced directory before delivering its change and preserve the records across rewatch",
  async run() {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync(os.tmpdir()), "rspack-directory-replacement-"),
    );
    const app = path.join(root, "app");
    const target = path.join(root, "target");
    const replacement = path.join(root, "replacement");
    for (const dir of [app, target, replacement]) fs.mkdirSync(dir);
    writeOld(path.join(target, "old.js"));
    writeOld(path.join(replacement, "new.js"));
    writeOld(path.join(replacement, "ignored.js"));
    for (const dir of [target, replacement])
      fs.mkdirSync(path.join(dir, "nested"));
    writeOld(path.join(target, "nested", "old.js"));
    writeOld(path.join(replacement, "nested", "new.js"));
    const entry = variant.link ? path.join(app, "link") : target;
    if (variant.link) fs.symlinkSync(target, entry, "junction");
    const directory = variant.nested ? app : entry;
    const oldFile = path.join(entry, "old.js");
    const newFile = path.join(entry, "new.js");
    const ignoredFile = path.join(entry, "ignored.js");
    const oldNestedFile = path.join(entry, "nested", "old.js");
    const newNestedFile = path.join(entry, "nested", "new.js");
    const watcher = new NativeWatcher({
      followSymlinks: variant.followSymlinks,
      aggregateTimeout: 40,
      ignored: (file) =>
        file === path.join(target, "ignored.js") || file === ignoredFile,
    });
    const changes = new Set();
    const removals = new Set();
    const snapshots = [];
    let failure;
    let replacing = false;
    let replacementTime;
    const records = () =>
      new Map(
        watcher
          .collectTimeInfoEntries()
          .fileTimestamps.entries.map((row) => [row.path, row]),
      );
    const watch = (initial) =>
      watcher.watch(
        [initial && variant.files ? [oldFile] : [], []],
        [initial ? [directory] : [], []],
        [initial && variant.files ? [newFile] : [], []],
        BigInt(Date.now()),
        (error, batch) => {
          if (error) failure = error;
          else {
            for (const file of batch.changedFiles) changes.add(file);
            for (const file of batch.removedFiles) removals.add(file);
            if (replacing && batch.changedFiles.includes(directory)) {
              snapshots.push(records());
            }
          }
        },
        () => {},
      );
    const waitFor = async (predicate) => {
      const deadline = Date.now() + 5000;
      while (!predicate() && !failure && Date.now() < deadline) await sleep(10);
      expect(failure).toBeUndefined();
      expect(predicate()).toBe(true);
    };
    const expectReplacement = (snapshot) => {
      expect(snapshot.has(oldFile)).toBe(false);
      expect(snapshot.has(newFile)).toBe(true);
      expect(snapshot.has(ignoredFile)).toBe(false);
      expect(snapshot.has(oldNestedFile)).toBe(false);
      expect(snapshot.has(newNestedFile)).toBe(true);
      expect(snapshot.get(newNestedFile).safeTime).toBeGreaterThanOrEqual(
        replacementTime,
      );
    };
    try {
      await watch(true);
      await waitFor(() => records().has(oldFile));
      await sleep(300);
      changes.clear();
      removals.clear();
      replacing = true;
      replacementTime = Date.now();
      fs.renameSync(target, path.join(root, "previous"));
      fs.renameSync(replacement, target);
      if (variant.rewatch) await watch(false);
      await waitFor(() => snapshots.length > 0);
      expectReplacement(snapshots[0]);
      if (variant.files) {
        await waitFor(() => removals.has(oldFile) && changes.has(newFile));
      } else {
        expect(removals.size).toBe(0);
      }
      await sleep(200);
      replacing = false;
      changes.clear();
      removals.clear();
      if (variant.gap) writeOld(path.join(target, "late.js"));
      await watch(false);
      await sleep(300);
      expectReplacement(records());
      if (variant.gap) {
        expect(records().has(path.join(entry, "late.js"))).toBe(true);
        await waitFor(() => changes.has(directory));
        changes.clear();
      } else {
        expect(changes.size).toBe(0);
      }
      expect(removals.size).toBe(0);
      fs.writeFileSync(path.join(target, "new.js"), "after rewatch");
      await waitFor(() => changes.has(directory));
      expectReplacement(records());
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
}));
