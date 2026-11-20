'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  target: ['web', 'es5'],
  optimization: {
    minimize: false,
    usedExports: true,
    concatenateModules: false,
  },
  plugins: [
    function apply() {
      this.hooks.compilation.tap('testcase', (compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          const source = assets['bundle0.js'].source();
          expect(source).toContain('unused:');
          expect(source).not.toMatch(/\.\.\.\s*void/);
        });
      });
    },
  ],
};
