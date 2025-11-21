const path = require("path");
const { createFsFromVolume, Volume } = require("memfs");

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig[]} */
module.exports = [
  (() => {
    const events = [];
    return {
      description: "should respect parallelism and dependencies for running",
      options(context) {
        return {
          parallelism: 1,
          2: {
            name: "c",
            context: path.join(__dirname, "fixtures"),
            entry: "./a.js",
            dependencies: ["d", "e"]
          },
          3: {
            name: "d",
            context: path.join(__dirname, "fixtures"),
            entry: "./a.js"
          },
          4: {
            name: "e",
            context: path.join(__dirname, "fixtures"),
            entry: "./a.js"
          }
        };
      },
      compiler(context, compiler) {
        compiler.compilers.forEach(c => {
          c.hooks.run.tap("test", () => {
            events.push(`${c.name} run`);
          });
          c.hooks.done.tap("test", () => {
            events.push(`${c.name} done`);
          });
        });
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          compiler.run((err, stats) => {
            expect(events.join(" ")).toBe(
              "a run a done b run b done d run d done e run e done c run c done"
            );
            compiler.close(resolve);
          });
        });
      }
    };
  })(),
  (() => {
    const watchCallbacks = [];
    const watchCallbacksUndelayed = [];
    const events = [];
    let update = 0;
    return {
      description: "should respect parallelism and dependencies for watching",
      options(context) {
        return Object.assign(
          {
            0: {
              name: "a",
              mode: "development",
              context: path.join(__dirname, "fixtures"),
              entry: "./a.js",
              dependencies: ["b", "c"]
            },
            1: {
              name: "b",
              mode: "development",
              context: path.join(__dirname, "fixtures"),
              entry: "./b.js"
            },
            2: {
              name: "c",
              mode: "development",
              context: path.join(__dirname, "fixtures"),
              entry: "./a.js"
            }
          },
          { parallelism: 1 }
        );
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
          }
        };
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
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          compiler.watch({}, (err, stats) => {
            if (err) return reject(err);
            const info = () =>
              stats.toString({ preset: "summary", version: false });
            switch (update++) {
              case 0:
                expect(info()).toMatchInlineSnapshot(`
					a:
					  a compiled with 1 error

					b:
					  b compiled with 1 error

					c:
					  c compiled with 1 error
				`);
                expect(compiler.compilers[0].modifiedFiles.size).toBe(0);
                expect(compiler.compilers[0].removedFiles.size).toBe(0);
                // CHANGE: Rspack generates a distinct snapshot
                expect(events).toMatchInlineSnapshot(`
					Array [
					  b run,
					  b done,
					  c run,
					  c done,
					  a run,
					  a done,
					]
				`);
                events.length = 0;
                // wait until watching begins
                setTimeout(() => {
                  watchCallbacksUndelayed[0]();
                  watchCallbacks[0](
                    null,
                    new Map(),
                    new Map(),
                    new Set(),
                    new Set()
                  );
                }, 100);
                break;
              case 1:
                expect(info()).toMatchInlineSnapshot(`
									a:
									  a compiled successfully

									b:
									  b compiled successfully
							`);
                expect(compiler.compilers[1].modifiedFiles.size).toBe(0);
                expect(compiler.compilers[1].removedFiles.size).toBe(0);
                // CHANGE: Rspack generates a distinct snapshot
                expect(events).toMatchInlineSnapshot(`
					Array [
					  b invalid,
					  b run,
					  b done,
					  a invalid,
					  a run,
					  a done,
					]
				`);
                watchCallbacksUndelayed[2]();
                watchCallbacks[2](
                  null,
                  new Map(),
                  new Map(),
                  new Set(),
                  new Set()
                );
                break;
              case 2:
                expect(info()).toMatchInlineSnapshot(`
									a:
									  a compiled successfully
							`);
                // CHANGE: Rspack generates a distinct snapshot
                expect(events).toMatchInlineSnapshot(`
					Array [
					  b invalid,
					  b run,
					  b done,
					  a invalid,
					  a run,
					  a done,
					  a invalid,
					  a run,
					  a done,
					]
				`);
                events.length = 0;
                watchCallbacksUndelayed[0]();
                watchCallbacksUndelayed[1]();
                watchCallbacks[0](
                  null,
                  new Map(),
                  new Map(),
                  new Set(),
                  new Set()
                );
                watchCallbacks[1](
                  null,
                  new Map(),
                  new Map(),
                  new Set(),
                  new Set()
                );
                break;
              case 3:
                expect(info()).toMatchInlineSnapshot(`
									a:
									  a compiled successfully

									b:
									  b compiled successfully

									c:
									  c compiled successfully
							`);
                // CHANGE: Rspack generates a distinct snapshot
                expect(events).toMatchInlineSnapshot(`
					Array [
					  b invalid,
					  c invalid,
					  b run,
					  b done,
					  c run,
					  c done,
					  a invalid,
					  a run,
					  a done,
					]
				`);
                events.length = 0;
                compiler.close(resolve);
                break;
              default:
                reject(new Error("unexpected"));
            }
          });
        });
      }
    };
  })(),
  (() => {
    const events = [];
    let state = 0;
    return {
      description: "should respect parallelism when using invalidate",
      options(context) {
        return Object.assign(
          {
            0: {
              name: "a",
              mode: "development",
              entry: { a: "./a.js" },
              context: path.join(__dirname, "fixtures")
            },
            1: {
              name: "b",
              mode: "development",
              entry: { b: "./b.js" },
              context: path.join(__dirname, "fixtures")
            }
          },
          { parallelism: 1 }
        );
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
				  a run,
				  a done,
				  b run,
				  b done,
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
					  a run,
					  a done,
					  b run,
					  b done,
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
  // fix issue #2585
  (() => {
    return {
      description: "should respect parallelism when using watching",
      options(context) {
        const configMaps = [];
        for (let index = 0; index < 3; index++) {
          configMaps.push({
            name: index.toString(),
            mode: "development",
            entry: "./src/main.jsx",
            devServer: {
              hot: true
            }
          });
        }
        configMaps.parallelism = 1;
        return configMaps;
      },
      async build(context, compiler) {
        return new Promise((resolve, reject) => {
          compiler.watch({}, err => {
            if (err) {
              compiler.close(() => {
                reject(err);
              });
              return;
            }
            compiler.close(err => {
              if (err) return reject(err);
              resolve();
            });
          });
        });
      }
    };
  })()
];
