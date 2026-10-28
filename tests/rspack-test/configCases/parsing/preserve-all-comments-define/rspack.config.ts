import { defineConfig, definePlugin } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new DefinePlugin({
      'process.env.__IS_REACT_18__': 'true',
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap('Test', (assets) => {
            let source = assets['bundle0.js'].source().toString();
            expect(source.match(/\/\* @__PURE__ \*\/ jsx/g) || []).toHaveLength(
              1,
            );
          });
        });
      },
    }),
  ],
});
