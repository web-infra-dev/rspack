import assert from 'node:assert';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

const { RawSource, ConcatSource } = rspack.sources;

export default defineConfig({
  plugins: [
    definePlugin({
      name: 'test',
      apply(compiler) {
        compiler.hooks.compilation.tap('compilation', (compilation) => {
          compilation.hooks.processAssets.tapPromise(
            'Test1',
            async (assets) => {
              for (const [key, value] of Object.entries(assets)) {
                compilation.updateAsset(
                  key,
                  new ConcatSource(new RawSource('//banner;\n'), value),
                );
              }
            },
          );

          compilation.hooks.processAssets.tapPromise(
            'Test2',
            async (assets) => {
              assert((Object.keys(assets).length = 1));
              assert((Object.getOwnPropertyNames(assets).length = 1));
              assert((Reflect.ownKeys(assets).length = 1));
              assert('bundle0.js' in assets);
              assert(
                assets['bundle0.js']
                  .source()
                  .toString()
                  .startsWith('//banner;\n'),
              );
            },
          );
        });
      },
    }),
  ],
});
