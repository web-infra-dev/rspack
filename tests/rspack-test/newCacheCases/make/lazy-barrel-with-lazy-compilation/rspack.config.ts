import { rspack } from '@rspack/core';
import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
const events = { built: 0, restored: 0 };

export default defineConfig({
  entry: './index.js',
  mode: 'development',
  devtool: false,
  incremental: false,
  experiments: { newCache: true },
  cache: { type: 'persistent' },
  // Cache restoration must not depend on checkout times or filesystem precision.
  snapshot: { module: { timestamp: false, hash: true } },
  lazyCompilation: { entries: false, imports: true },
  module: { rules: [{ test: /barrel\.js$/, sideEffects: false }] },
  optimization: { usedExports: false, sideEffects: 'flag' },
  plugins: [
    definePlugin({
      apply(compiler) {
        rspack.lazyCompilationMiddleware(compiler);
        const isBarrel = (id: string) =>
          id.replace(/[\\/]/g, '/').endsWith('/barrel.js');
        compiler.hooks.compilation.tap('LazyBarrel', (compilation) => {
          compilation.hooks.buildModule.tap('LazyBarrel', (module) => {
            if (isBarrel(module.identifier())) events.built++;
          });
          compilation.hooks.stillValidModule.tap('LazyBarrel', (module) => {
            if (isBarrel(module.identifier())) events.restored++;
          });
        });
        compiler.hooks.done.tap('LazyBarrel', (stats) => {
          expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
          expect(events).toEqual(
            buildIndex === 0
              ? { built: 1, restored: 0 }
              : { built: 0, restored: 1 },
          );
          expect(
            [...stats.compilation.modules].some((module) =>
              module.identifier().endsWith('/extra.js'),
            ),
          ).toBe(false);
          events.built = 0;
          events.restored = 0;
          buildIndex++;
        });
      },
    }),
  ],
});
