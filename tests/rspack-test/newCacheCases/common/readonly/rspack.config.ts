import { defineConfig, definePlugin } from '@rspack/cli';
import type {
  FileSystemCacheOptions,
  PersistentCacheOptions,
} from '@rspack/core';

const loaderOptions = { count: 0 };
let index = 0;

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
  },
  module: {
    rules: [
      {
        test: /file\.js$/,
        use: {
          loader: './loader.mjs',
          options: loaderOptions,
        },
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const cache = compiler.options.cache as
          PersistentCacheOptions | FileSystemCacheOptions;
        let shouldRebuildFile = true;
        if (index == 0) {
          cache.readonly = false;
          shouldRebuildFile = true;
        } else if (index == 1) {
          cache.readonly = true;
          shouldRebuildFile = true;
        } else if (index == 2) {
          cache.readonly = true;
          shouldRebuildFile = false;
        } else if (index == 3) {
          cache.readonly = false;
          shouldRebuildFile = true;
        } else if (index == 4) {
          cache.readonly = true;
          shouldRebuildFile = false;
        }

        compiler.hooks.done.tap('PLUGIN', function () {
          if (shouldRebuildFile) {
            expect(loaderOptions.count).toBe(1);
          } else {
            expect(loaderOptions.count).toBe(0);
          }
          // reset
          loaderOptions.count = 0;
          index++;
        });
      },
    }),
  ],
});
