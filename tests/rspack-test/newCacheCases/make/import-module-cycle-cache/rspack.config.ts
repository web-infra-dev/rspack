import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;

export default defineConfig({
  optimization: { minimize: false },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckCaughtCycleCache', (stats) => {
          if (buildIndex++ !== 1) return;
          const info = stats.toJson({ all: false, cacheInfo: true }).cacheInfo![
            '|Compilation/modules'
          ];
          expect(info.hits).toBeGreaterThan(0);
          expect(info.misses).toBe(0);
          expect(info.reused).toBe(info.hits);
        });
      },
    }),
  ],
});
