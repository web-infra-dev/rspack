import { defineConfig, definePlugin } from '@rspack/cli';

let firstBuild = true;

export default defineConfig({
  mode: 'none',
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('PartialHitTest', (stats) => {
          if (firstBuild) {
            firstBuild = false;
            return;
          }

          const cacheInfo = stats.toJson({
            all: false,
            cacheInfo: true,
          }).cacheInfo!;
          expect(
            cacheInfo[`${compiler.compilerPath}|Compilation/codeGeneration`],
          ).toEqual({ hits: 2, misses: 1, reused: 2, stores: 1 });
          expect(
            cacheInfo[`${compiler.compilerPath}|Compilation/modules`],
          ).toEqual({ hits: 3, misses: 0, reused: 2, stores: 1 });
        });
      },
    }),
  ],
});
