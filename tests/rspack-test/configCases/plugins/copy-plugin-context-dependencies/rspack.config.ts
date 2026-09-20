import { defineConfig, definePlugin } from '@rspack/cli';

import { CopyRspackPlugin } from '@rspack/core';
import path from 'node:path';
export default defineConfig({
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          // A glob `from` registers the closest common parent dir of the matched
          // files as a context dependency. On Windows the glob matcher yields
          // forward-slash paths, so the registered dir must be normalized back to
          // native separators to stay consistent with the rest of the dep graph.
          from: './public/**/*',
        },
      ],
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          for (const dir of stats.compilation.contextDependencies) {
            expect(dir).toBe(path.normalize(dir));
          }
        });
      },
    }),
  ],
});
