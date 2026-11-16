import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(path.resolve("tests/rspack-test/package.json"));
const core = require("@rspack/core");
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
  "@rspack/binding",
);
const Watchpack = require(
  path.join(
    path.dirname(require.resolve("@rspack/core/package.json")),
    "compiled/watchpack",
  ),
);
await core.experiments.globalTrace.register(
  "rspack_watcher::fs_event=debug",
  "logger",
  "stdout",
);

for (const native of [false, true]) {
  for (const replaceLink of [false, true]) {
    const root = fs.mkdtempSync(
      path.join(fs.realpathSync.native(os.tmpdir()), "rspack-ignore-trace-"),
    );
    const app = path.join(root, "app");
    const target = path.join(root, "target");
    const outside = path.join(root, "outside");
    for (const directory of [app, target, outside]) fs.mkdirSync(directory);
    const link = path.join(app, "link");
    const real = path.join(target, "value.js");
    const alias = path.join(link, "value.js");
    const a = path.join(outside, "a.js");
    const b = path.join(outside, "b.js");
    fs.writeFileSync(a, "a");
    fs.writeFileSync(b, "b");
    if (replaceLink) fs.symlinkSync(a, real, "file");
    else fs.writeFileSync(real, "before");
    fs.symlinkSync(target, link, "junction");
    await sleep(2200);
    const label = `${native ? "native" : "watchpack"}-${replaceLink ? "link-replacement" : "file-restore"}`;
    const log = (type, details) =>
      console.log(JSON.stringify({ label, type, ...details }));
    const watcher = new (native ? NativeWatcher : Watchpack)({
      followSymlinks: true,
      aggregateTimeout: 50,
      ignored(entry) {
        const ignored = entry === alias;
        log("ignored", { entry, ignored });
        return ignored;
      },
    });
    const files = replaceLink ? [] : [alias];
    const directories = replaceLink ? [app, link] : [app];
    const record = (changed, removed) =>
      log("aggregated", { changed: [...changed], removed: [...removed] });
    log("start", { root, alias, directories });
    try {
      if (native) {
        watcher.watch(
          [files, []],
          [directories, []],
          [[], []],
          BigInt(Date.now()),
          (error, batch) => {
            if (error) throw error;
            record(batch.changedFiles, batch.removedFiles);
          },
          (file, time) => log("undelayed", { file, time: String(time) }),
        );
      } else {
        watcher.on("aggregated", record);
        watcher.watch({ files, directories, startTime: Date.now() });
      }
      await sleep(1000);
      if (replaceLink) {
        log("replace-link", {});
        fs.unlinkSync(real);
        fs.symlinkSync(b, real, "file");
      } else {
        log("edit-file", {});
        fs.writeFileSync(real, "changed");
        await sleep(1000);
        log("remove-file", {});
        fs.unlinkSync(real);
        await sleep(1000);
        log("restore-file", {});
        fs.writeFileSync(real, "restored");
      }
      await sleep(1000);
      log("done", {});
    } finally {
      await watcher.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
}
