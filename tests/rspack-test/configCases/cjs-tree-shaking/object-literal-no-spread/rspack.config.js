'use strict';

const assertNoObjectSpread = (assetName) =>
  function apply() {
    this.hooks.compilation.tap('testcase', (compilation) => {
      compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
        const source = assets[assetName].source().toString();
        expect(source).toContain('unused:');
        expect(source).not.toMatch(/\.\.\.\s*void/);
      });
    });
  };

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
  plugins: [assertNoObjectSpread('bundle0.js')],
};
