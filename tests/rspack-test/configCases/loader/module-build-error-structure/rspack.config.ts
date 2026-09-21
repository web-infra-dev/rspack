import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.js$/i,
        use: [{ loader: './loader.mjs' }],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('PLUGIN', (stats) => {
          const { errors } = stats.compilation;
          expect(errors).toHaveLength(1);

          const error = errors[0];
          expect(error).toMatchObject({
            name: 'ModuleBuildError',
            error: {
              name: 'NextFontError',
            },
          });
          assert(error.error);
          expect(error.error.message).toContain(
            'Cannot be used within pages/_document.js',
          );
        });
      },
    }),
  ],
});
