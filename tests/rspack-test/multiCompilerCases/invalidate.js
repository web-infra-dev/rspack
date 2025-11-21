const { createFsFromVolume, Volume } = require("memfs");
const path = require("path");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [
  (() => {
    const events = [];
    let state = 0;
    return {
      description: "should respect dependencies when using invalidate",
      options(context) {
        return {
          0: {
            name: "a",
            mode: "development",
            entry: { a: "./a.js" },
            context: path.join(__dirname, "fixtures"),
            dependencies: ["b"]
          },
          1: {
            name: "b",
            mode: "development",
            entry: { b: "./b.js" },
            context: path.join(__dirname, "fixtures")
          }
        };
      },
      compiler(context, compiler) {
        compiler.compilers.forEach(c => {
          c.hooks.invalid.tap("test", () => {
            events.push(`${c.name} invalid`);
          });
          c.hooks.watchRun.tap("test", () => {
            events.push(`${c.name} run`);
          });
          c.hooks.done.tap("test", () => {
            events.push(`${c.name} done`);
          });
        });

        compiler.watchFileSystem = { watch() { } };
        compiler.outputFileSystem = createFsFromVolume(new Volume());
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          const watching = compiler.watch({}, error => {
            if (error) {
              reject(error);
              return;
            }
            if (state !== 0) return;
            state++;

            // CHANGE: Rspack generates a distinct snapshot
            expect(events).toMatchInlineSnapshot(`
				Array [
				  b run,
				  b done,
				  a run,
				  a done,
				]
			`);
            events.length = 0;

            watching.invalidate(err => {
              try {
                if (err) return reject(err);

                // CHANGE: Rspack generates a distinct snapshot
                expect(events).toMatchInlineSnapshot(`
					Array [
					  a invalid,
					  b invalid,
					  b run,
					  b done,
					  a run,
					  a done,
					]
				`);
                events.length = 0;
                expect(state).toBe(1);
                setTimeout(() => {
                  compiler.close(resolve);
                }, 1000);
              } catch (e) {
                console.error(e);
                reject(e);
              }
            });
          });
        });
      }
    };
  })(),
  (() => {
    const entriesA = { a: "./a.js" };
    const entriesB = { b: "./b.js" };
    return {
      description: "shouldn't hang when invalidating watchers",
      options(context) {
        return Object.assign({
          0: {
            name: "a",
            mode: "development",
            entry: () => entriesA,
            context: path.join(__dirname, "fixtures")
          },
          1: {
            name: "b",
            mode: "development",
            entry: () => entriesB,
            entry: entriesB,
            context: path.join(__dirname, "fixtures")
          }
        });
      },
      compiler(context, compiler) {
        compiler.watchFileSystem = { watch() { } };
        compiler.outputFileSystem = createFsFromVolume(new Volume());
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          const watching = compiler.watch({}, error => {
            if (error) {
              reject(error);
              return;
            }

            entriesA.b = "./b.js";
            entriesB.a = "./a.js";

            watching.invalidate(err => {
              if (err) return reject(err);
              compiler.close(resolve);
            });
          });
        });
      }
    };
  })(),
  (() => {
    const watchCallbacks = [];
    const watchCallbacksUndelayed = [];
    let firstRun = true;
    return {
      description: "shouldn't hang when invalidating during build",
      options(context) {
        return {
          0: {
            name: "a",
            mode: "development",
            context: path.join(__dirname, "fixtures"),
            entry: "./a.js"
          },
          1: {
            name: "b",
            mode: "development",
            context: path.join(__dirname, "fixtures"),
            entry: "./b.js",
            dependencies: ["a"]
          }
        };
      },
      compiler(context, compiler) {
        compiler.outputFileSystem = createFsFromVolume(new Volume());
        compiler.watchFileSystem = {
          watch(
            files,
            directories,
            missing,
            startTime,
            options,
            callback,
            callbackUndelayed
          ) {
            watchCallbacks.push(callback);
            watchCallbacksUndelayed.push(callbackUndelayed);
            if (firstRun && files.has(path.join(__dirname, "fixtures", "a.js"))) {
              process.nextTick(() => {
                callback(null, new Map(), new Map(), new Set(), new Set());
              });
              firstRun = false;
            }

            // watch should return a watcher instance
            // watcher instance should have close, pause and getInfo methods
            return {
              close: () => { },
              pause: () => { },
              getInfo: () => {
                return {
                  changes: new Set(),
                  removals: new Set(),
                  fileTimeInfoEntries: new Map(),
                  directoryTimeInfoEntries: new Map(),
                };
              }
            }
          }
        };
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          compiler.watch({}, (err, stats) => {
            if (err) return reject(err);
            compiler.close(resolve);
          });
        });
      }
    };
  })(),
];
