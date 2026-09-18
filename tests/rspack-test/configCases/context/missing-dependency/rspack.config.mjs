import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          expect(Array.from(stats.compilation.missingDependencies)).toContain(
            path.resolve(import.meta.dirname, './lang'),
          );
        });
      },
    },
  ],
};
