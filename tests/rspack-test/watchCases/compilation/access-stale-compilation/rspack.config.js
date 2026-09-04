const TOTAL_BUILDS = 3;

let build = 0;
let staleCompilation = null;
let staleRead = null;
let midMakeReads = null;

function capture(read) {
  try {
    return { ok: true, value: read() };
  } catch (err) {
    return { ok: false, message: String(err && err.message) };
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap('PLUGIN', (nmf) => {
          nmf.hooks.afterResolve.tap('PLUGIN', () => {
            // The module graph artifact is stolen for the duration of the make phase, so
            // accessors that reach into it have to report that rather than dereference the
            // stolen cell and abort the process.
            if (!midMakeReads) {
              midMakeReads = {
                modules: capture(() => compiler._lastCompilation.modules.size),
                builtModules: capture(
                  () => compiler._lastCompilation.builtModules.size,
                ),
              };
            }
          });
        });

        compiler.hooks.make.tap('PLUGIN', () => {
          if (!staleCompilation || staleRead) return;
          // Read a Compilation the compiler has already replaced, from a timer, i.e. from
          // the JS thread with no Rust -> JS tap on the stack. That is what a dev server
          // does when it answers a request while a rebuild is running.
          setTimeout(() => {
            staleRead = capture(() => staleCompilation.modules.size);
          });
        });

        compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
          build++;
          compilation.hooks.seal.tap('PLUGIN', () => {
            // The live Compilation stays fully readable once the artifact is back in its
            // cell, so a guard that simply rejected everything would not pass here.
            expect(compilation.modules.size).toBeGreaterThan(0);

            expect(midMakeReads.modules.ok).toBe(false);
            expect(midMakeReads.modules.message).toContain(
              'ModuleGraph is not available',
            );
            expect(midMakeReads.builtModules.ok).toBe(false);

            if (build < TOTAL_BUILDS) return;

            // Asserted unconditionally on the last build: if the timer never ran, this
            // fails instead of letting the case pass without checking anything.
            expect(staleRead).not.toBe(null);
            expect(staleRead.ok).toBe(false);
            expect(staleRead.message).toContain(
              'Unable to access compilation with id',
            );
          });
        });

        compiler.hooks.done.tap('PLUGIN', (stats) => {
          if (!staleCompilation) {
            staleCompilation = stats.compilation;
          }
        });
      },
    },
  ],
};
