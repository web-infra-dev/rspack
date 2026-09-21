import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  cache: {
    type: 'filesystem',
  },
  module: {
    rules: [
      {
        test: /\.generate-json\.js$/,
        use: './loader',
        type: 'json',
      },
    ],
  },
  plugins: [
    new rspack.ProgressPlugin(),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tapPromise('CacheTest', async () => {
          const cache = compiler
            .getCache('ProgressPlugin')
            .getItemCache('counts', null);

          const data = await cache.getPromise();
          assert(typeof data === 'object' && data !== null);
          assert('modulesCount' in data);
          assert('dependenciesCount' in data);

          if (data.modulesCount !== 3) {
            throw new Error(
              `Wrong cached value of \`ProgressPlugin.modulesCount\` - ${data.modulesCount}, expect 3`,
            );
          }

          if (data.dependenciesCount !== 3) {
            throw new Error(
              `Wrong cached value of \`ProgressPlugin.dependenciesCount\` - ${data.dependenciesCount}, expect 3`,
            );
          }
        });
      },
    }),
  ],
});
