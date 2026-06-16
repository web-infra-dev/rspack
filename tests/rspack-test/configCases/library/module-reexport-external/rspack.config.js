'use strict';

/** @typedef {import("@rspack/core").Compilation} Compilation */

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'none',
  entry: { main: './index.js', test: './test' },
  output: {
    module: true,
    library: {
      type: 'module',
    },
    filename: '[name].js',
    chunkFormat: 'module',
  },
  resolve: {
    extensions: ['.js'],
  },
  externalsType: 'module',
  externals: ['external0'],
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    function apply() {
      /**
       * @param {Compilation} compilation compilation
       */
      const handler = (compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          const source = assets['test.js'].source();
          const exportsName = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
            ? '__rspack_exportsvalue'
            : '__webpack_exports__value';
          expect(source).toContain(`export { ${exportsName} as value };`);
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    },
  ],
};
