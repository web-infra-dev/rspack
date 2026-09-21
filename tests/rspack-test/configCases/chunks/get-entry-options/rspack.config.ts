import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    main: {
      import: './index.js',
      chunkLoading: 'async-node',
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('test', (compilation) => {
          compilation.hooks.afterSeal.tap('test', () => {
            let entrypoint = compilation.entrypoints.get('main');
            assert(entrypoint);

            entrypoint.chunks.forEach((chunk) => {
              const entryOptions = chunk.getEntryOptions();

              expect(entryOptions).not.toBeUndefined();
              assert(entryOptions);
              expect(entryOptions.chunkLoading).toBe('async-node');
            });
          });
        });
      },
    }),
  ],
});
