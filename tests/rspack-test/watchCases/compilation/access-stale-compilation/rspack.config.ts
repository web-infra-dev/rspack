import { defineConfig, definePlugin } from '@rspack/cli';
import { type Chunk, type Compilation } from '@rspack/core';

const TOTAL_BUILDS = 3;

let build = 0;
let staleCompilation: Compilation | null = null;
let staleRead: ReturnType<typeof capture> | null = null;
let staleChunks: ReadonlySet<Chunk> | null = null;
let midMakeReads: {
  modules: ReturnType<typeof capture>;
  builtModules: ReturnType<typeof capture>;
} | null = null;

function capture(read: () => number) {
  try {
    return { ok: true, value: read() };
  } catch (err) {
    return {
      ok: false,
      message: String(err instanceof Error ? err.message : err),
    };
  }
}

export default defineConfig({
  incremental: { buildChunkGraph: true },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.normalModuleFactory.tap('PLUGIN', (nmf) => {
          nmf.hooks.afterResolve.tap('PLUGIN', () => {
            // The module graph artifact is stolen for the duration of the make phase, so
            // accessors that reach into it have to report that rather than dereference the
            // stolen cell and abort the process.
            if (!midMakeReads) {
              midMakeReads = {
                modules: capture(() => compiler._lastCompilation!.modules.size),
                builtModules: capture(
                  () => compiler._lastCompilation!.builtModules.size,
                ),
              };
            }
          });
        });

        compiler.hooks.make.tap('PLUGIN', (compilation) => {
          if (staleChunks) {
            // A chunk set saved in done follows the compiler's current build.
            expect(staleChunks.size).toBe(compilation.chunks.size);
            expect(staleChunks.has({} as Chunk)).toBe(false);
          }
          if (!staleCompilation || staleRead) return;
          // Read a Compilation the compiler has already replaced, from a timer, i.e. from
          // the JS thread with no Rust -> JS tap on the stack. That is what a dev server
          // does when it answers a request while a rebuild is running.
          setTimeout(() => {
            staleRead = capture(() => staleCompilation!.modules.size);
          });
        });

        compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
          build++;
          compilation.hooks.seal.tap('PLUGIN', () => {
            // The live Compilation stays fully readable once the artifact is back in its
            // cell, so a guard that simply rejected everything would not pass here.
            expect(compilation.modules.size).toBeGreaterThan(0);

            expect(midMakeReads?.modules.ok).toBe(false);
            expect(midMakeReads?.modules.message).toContain(
              'ModuleGraph is not available',
            );
            expect(midMakeReads?.builtModules.ok).toBe(false);

            if (build < TOTAL_BUILDS) return;

            // Asserted unconditionally on the last build: if the timer never ran, this
            // fails instead of letting the case pass without checking anything.
            expect(staleRead).not.toBe(null);
            expect(staleRead?.ok).toBe(false);
            expect(staleRead?.message).toContain(
              'Unable to access compilation with id',
            );
          });
        });

        compiler.hooks.done.tap('PLUGIN', (stats) => {
          if (staleChunks) {
            const current = Array.from(stats.compilation.chunks);
            const retained = Array.from(staleChunks);
            expect(staleChunks.size).toBe(current.length);
            expect(retained.length).toBe(current.length);
            for (const [index, chunk] of current.entries()) {
              // IDs can survive incremental rebuilds, but wrappers must belong
              // to the current compilation even when the cached IDs match.
              expect(retained[index]).toBe(chunk);
              expect(staleChunks.has(chunk)).toBe(true);
              expect(Array.from(retained[index].files)).toEqual(
                Array.from(chunk.files),
              );
            }
          }
          if (!staleCompilation) {
            staleCompilation = stats.compilation;
            staleChunks = stats.compilation.chunks;
            expect(staleChunks.size).toBeGreaterThan(0);
            // Warm the native array cache before the compilation is replaced.
            expect(Array.from(staleChunks).length).toBe(staleChunks.size);
          }
        });
      },
    }),
  ],
});
