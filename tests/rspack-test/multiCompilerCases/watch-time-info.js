const path = require("node:path");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = ["direct", "wrapped", "copied-properties"].map(mode => {
  const states = [];
  return {
    description: `should read current watch timestamps when scheduled (${mode})`,
    options() {
      return ["a", "b"].map(name => ({
        name,
        context: path.resolve(__dirname, "../fixtures"),
        entry: `./${name}.js`,
        mode: "development",
        experiments: { nativeWatcher: false },
        plugins: [{
          apply(compiler) {
            // The multi-compiler harness replaces watchFileSystem after setup.
            states.push({ compiler, fs: compiler.watchFileSystem, sweeps: 0, callbacks: 0 });
          }
        }]
      }));
    },
    compiler() {
      for (const state of states) {
        const { compiler, fs } = state;
        compiler.watchFileSystem = fs;
        const watch = fs.watch.bind(fs);
        fs.watch = (...args) => {
          const callback = args[5];
          if (mode !== "direct") {
            const wrapped = (...values) => {
              expect(values[1]).toBeInstanceOf(Map);
              expect(values[2]).toBeInstanceOf(Map);
              state.callbacks++;
              return callback(...values);
            };
            if (mode === "copied-properties") {
              Object.defineProperties(wrapped, Object.getOwnPropertyDescriptors(callback));
            }
            args[5] = wrapped;
          }
          const handle = watch(...args);
          const watcher = fs.watcher;
          const collect = watcher.collectTimeInfoEntries.bind(watcher);
          watcher.collectTimeInfoEntries = (files, directories) => {
            state.sweeps++;
            collect(files, directories);
            files.set(state.late, { safeTime: state.generation });
            directories.set(compiler.context, { safeTime: state.generation });
          };
          return handle;
        };
      }
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
            // Watching rearms on nextTick after delivering the build result.
            setImmediate(() => {
              try {
                for (const state of states) {
                  const { compiler, fs } = state;
                  state.first = path.join(compiler.context, "time-info-first.js");
                  state.late = path.join(compiler.context, "time-info-late.js");
                  state.generation = 1;
                  const watcher = fs.watcher;
                  // Deliver a batch synchronously, leaving MultiCompiler queued.
                  watcher._onChange(state.first, 1, state.first, "change");
                  clearTimeout(watcher.aggregateTimer);
                  watcher._onTimeout();
                  expect(state.sweeps).toBe(mode === "direct" ? 0 : 1);

                  // Paused Watchpack still records events. The next build must
                  // consume these changes and the updated timestamp generation.
                  state.generation = 2;
                  watcher._onChange(state.late, 2, state.late, "change");
                  watcher._onRemove(state.first, state.first, "rename");
                  compiler.hooks.watchRun.tap("WatchTimeInfoTest", () => {
                    try {
                      expect(state.sweeps).toBe(mode === "direct" ? 1 : 2);
                      expect(compiler.fileTimestamps.get(state.late)).toEqual({ safeTime: 2 });
                      expect(compiler.contextTimestamps.get(compiler.context)).toEqual({ safeTime: 2 });
                      expect(compiler.modifiedFiles.has(state.late)).toBe(true);
                      expect(compiler.modifiedFiles.has(state.first)).toBe(false);
                      expect(compiler.removedFiles.has(state.first)).toBe(true);
                      expect(state.callbacks).toBe(mode === "direct" ? 0 : 1);
                    } catch (error) {
                      reject(error);
                    }
                  });
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
  };
});
