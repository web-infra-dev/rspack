import { CopyRspackPlugin } from '@rspack/core';
import path from 'node:path';
export default {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: './public',
        },
      ],
    }),
    {
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          for (const file of stats.compilation.fileDependencies) {
            // Verify that fileDependencies are always normalized
            expect(file).toBe(path.normalize(file));
          }
        });
      },
    },
  ],
};
