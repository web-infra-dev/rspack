import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('DonePlugin', (stats) => {
          expect(Array.from(stats.compilation.missingDependencies)).toContain(
            path.resolve(import.meta.dirname, './lang'),
          );
        });
      },
    }),
  ],
});
