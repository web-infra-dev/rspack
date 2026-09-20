import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const { ConcatSource } = rspack.sources;

export default defineConfig({
  entry: {
    main: './index.js',
  },
  plugins: [
    definePlugin({
      name: 'test',
      apply(compiler) {
        compiler.hooks.compilation.tap('compilation', (compilation) => {
          compilation.hooks.processAssets.tapPromise(
            'processAssets1',
            async (_assets) => {
              let inspect = new ConcatSource();
              for (let [n, cg] of compilation.entrypoints) {
                inspect.add(`entry name: ${n}\n`);
                for (let file of cg.getFiles()) {
                  inspect.add(`  file: ${file}\n`);
                }
              }
              compilation.emitAsset('inspect.txt', inspect);
            },
          );
        });
      },
    }),
  ],
});
