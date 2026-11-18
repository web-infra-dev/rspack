import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [{ test: /a\.js$/, use: ['./query-loader.mjs'] }],
  },
  plugins: [
    new rspack.ContextReplacementPlugin(
      /modules$/,
      path.join(import.meta.dirname, 'modules?cats=meow#dogs'),
      { './a.js': path.join(import.meta.dirname, 'modules/a.js') },
    ),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CheckContextResourcePath', (stats) => {
          const dependencies = [...stats.compilation.contextDependencies];
          expect(dependencies).toContain(
            path.join(compiler.context, 'modules'),
          );
          expect(
            dependencies.some(
              (value) =>
                value.includes('?cats=meow') || value.includes('#dogs'),
            ),
          ).toBe(false);
        });
      },
    }),
  ],
});
