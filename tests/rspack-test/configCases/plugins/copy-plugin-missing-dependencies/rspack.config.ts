import { defineConfig, definePlugin } from '@rspack/cli';
import { CopyRspackPlugin } from '@rspack/core';
import path from 'node:path';

const missingDir = path.join(import.meta.dirname, 'public');

export default defineConfig({
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: '**/*',
          context: missingDir,
          noErrorOnMissing: true,
        },
      ],
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          // A glob base dir that does not exist must be a missing dependency, so
          // watch mode does not report it as removed and recompile right away.
          expect(stats.compilation.missingDependencies.has(missingDir)).toBe(
            true,
          );
          expect(stats.compilation.contextDependencies.has(missingDir)).toBe(
            false,
          );
        });
      },
    }),
  ],
});
