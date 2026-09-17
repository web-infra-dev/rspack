import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new DefinePlugin({
      'process.env.__IS_REACT_18__': 'true',
    }),
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap('Test', (assets) => {
            let source = assets['bundle0.js'].source();
            expect(source.match(/\/\* @__PURE__ \*\/ jsx/g) || []).toHaveLength(
              1,
            );
          });
        });
      },
    },
  ],
};
