import { defineConfig, definePlugin } from '@rspack/cli';
import type {
  FileSystemCacheOptions,
  PersistentCacheOptions,
} from '@rspack/core';
import path from 'node:path';

const versions = ['v1', 'v2', 'v1'];
let buildIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(import.meta.dirname, 'immutable')],
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const cache = compiler.options.cache as
          PersistentCacheOptions | FileSystemCacheOptions;
        compiler.hooks.beforeCompile.tap('Test Plugin', function () {
          const version = versions[buildIndex++];
          if (version) {
            cache.version = version;
          }
        });
      },
    }),
  ],
});
