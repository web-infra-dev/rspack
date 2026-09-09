/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
  "success",
  "sync exception",
  "promise rejection",
  "callback error",
  "invalid result",
  "invalid promise result",
].map((mode) => {
  let completedPromises;
  let resolvedModules;
  return {
    description: `preserves JS tap ${mode}`,
    options() {
      return {
        entry: "./d",
        incremental: false,
        plugins: [
          (compiler) => {
            compiler.hooks.compilation.tap(
              "JsTapResults",
              (_compilation, params) => {
                params.normalModuleFactory.hooks.beforeResolve.tapPromise(
                  "JsTapResults",
                  async () => {
                    await Promise.resolve();
                    resolvedModules++;
                    if (mode === "invalid promise result") {
                      return "not a boolean";
                    }
                  },
                );
              },
            );
            compiler.hooks.make.tapPromise("JsTapResults", async () => {
              await new Promise((resolve) => setImmediate(resolve));
              if (mode === "promise rejection") {
                throw new Error("JS tap promise rejection");
              }
              completedPromises++;
            });
            compiler.hooks.make.tapAsync(
              "JsTapResultsCallback",
              (_compilation, callback) => {
                setImmediate(() =>
                  callback(
                    mode === "callback error"
                      ? new Error("JS tap callback error")
                      : undefined,
                  ),
                );
              },
            );
            compiler.hooks.shouldEmit.tap("JsTapResults", () => {
              if (mode === "sync exception") {
                throw new Error("JS tap sync exception");
              }
              if (mode === "invalid result") {
                return "not a boolean";
              }
              return true;
            });
          },
        ],
      };
    },
    async build(_context, compiler) {
      // Successful builds exercise fresh compilation closures and cached taps.
      // Fatal-error cases use separate compilers rather than restarting while the
      // native completion callback is still being finalized.
      for (let run = 0; run < (mode === "success" ? 3 : 1); run++) {
        completedPromises = 0;
        resolvedModules = 0;
        const { err, stats } = await new Promise((resolve) => {
          compiler.run((err, stats) => resolve({ err, stats }));
        });
        if (mode === "success") {
          expect(err).toBeFalsy();
          expect(stats.hasErrors()).toBe(false);
          expect(completedPromises).toBe(1);
          expect(resolvedModules).toBeGreaterThan(0);
        } else if (mode === "invalid promise result") {
          expect(err).toBeFalsy();
          expect(stats.hasErrors()).toBe(true);
          expect(stats.toString({ all: false, errors: true })).toContain(
            "bool",
          );
        } else {
          expect(err).toBeTruthy();
          expect(err.message).toContain(
            mode === "invalid result" ? "boolean" : `JS tap ${mode}`,
          );
        }
      }
    },
  };
});
