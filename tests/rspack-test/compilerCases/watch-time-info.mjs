import path from "node:path";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
const cases = [false, true].map(suspended => ({
  description: `should read watch timestamps once ${suspended ? "after resuming" : "when starting immediately"}`,
  options() {
    return { experiments: { nativeWatcher: false } };
  },
  async build(context, compiler) {
    let watching;
    let builds = 0;
    try {
      await new Promise((resolve, reject) => {
        watching = compiler.watch({ aggregateTimeout: 10000 }, (error, stats) => {
          if (error) return reject(error);
          if (stats.hasErrors()) return reject(new Error(stats.toString()));
          if (++builds === 2) return resolve();
          if (builds !== 1) return;
          setImmediate(() => {
            try {
              const watcher = compiler.watchFileSystem.watcher;
              const file = path.join(compiler.context, "time-info-change.js");
              let sweeps = 0;
              let generation = 1;
              const collect = watcher.collectTimeInfoEntries.bind(watcher);
              watcher.collectTimeInfoEntries = (files, directories) => {
                sweeps++;
                collect(files, directories);
                files.set(file, { safeTime: generation });
              };
              compiler.hooks.watchRun.tap("WatchTimeInfoTest", () => {
                try {
                  expect(sweeps).toBe(1);
                  expect(compiler.fileTimestamps.get(file)).toEqual({ safeTime: suspended ? 2 : 1 });
                  expect(compiler.modifiedFiles.has(file)).toBe(true);
                } catch (error) {
                  reject(error);
                }
              });
              if (suspended) watching.suspend();
              watcher._onChange(file, 1, file, "change");
              clearTimeout(watcher.aggregateTimer);
              watcher._onTimeout();
              if (suspended) {
                expect(sweeps).toBe(0);
                generation = 2;
                watching.resume();
              }
            } catch (error) {
              reject(error);
            }
          });
        });
      });
    } finally {
      if (watching) {
        await new Promise((resolve, reject) => watching.close(error => error ? reject(error) : resolve()));
      }
    }
  }
}));

cases.push({
  description: "should preserve timestamp Maps for direct watchFileSystem.watch callers",
  options() {
    return { experiments: { nativeWatcher: false } };
  },
  build(context, compiler) {
    const fs = compiler.watchFileSystem;
    const file = path.join(compiler.context, "time-info-direct.js");
    let calls = 0;
    let sweeps = 0;
    const handle = fs.watch(
      [], [], [], Date.now(), { aggregateTimeout: 10000 },
      (error, files, directories, changes, removals) => {
        calls++;
        expect(error).toBeNull();
        expect(sweeps).toBe(1);
        expect(files).toBeInstanceOf(Map);
        expect(directories).toBeInstanceOf(Map);
        expect(files.get(file)).toEqual({ safeTime: 1 });
        expect(directories.get(compiler.context)).toEqual({ safeTime: 1 });
        expect(changes).toEqual(new Set([file]));
        expect(removals).toEqual(new Set());
      },
      () => {}
    );
    try {
      const watcher = fs.watcher;
      const collect = watcher.collectTimeInfoEntries.bind(watcher);
      watcher.collectTimeInfoEntries = (files, directories) => {
        sweeps++;
        collect(files, directories);
        files.set(file, { safeTime: 1 });
        directories.set(compiler.context, { safeTime: 1 });
      };
      watcher._onChange(file, 1, file, "change");
      clearTimeout(watcher.aggregateTimer);
      watcher._onTimeout();
      expect(calls).toBe(1);
    } finally {
      handle.close();
    }
  }
});

export default cases;
