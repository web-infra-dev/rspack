/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: {
    index: './index.js',
  },
  output: {
    filename: `[name].js`,
    module: true,
    library: { type: 'modern-module' },
    iife: false,
    chunkFormat: 'module',
  },
  externalsType: 'module-import',
  externals: 'external-module',
  optimization: {
    runtimeChunk: false,
  },
  plugins: [
    function () {
      /**
       * @param {import("@rspack/core").Compilation} compilation compilation
       * @returns {void}
       */
      const handler = (compilation) => {
        compilation.hooks.afterProcessAssets.tap('testcase', (assets) => {
          const bundle = Object.values(assets)[0]._value;
          const interopHelper = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
            ? 'compatGetDefaultExport'
            : '__webpack_require__\\.n';
          const interop = bundle.match(
            new RegExp(
              `var (\\w+) = /\\*#__PURE__\\*/${interopHelper}\\((\\w+)\\);\\nvar (\\w+) = \\1\\(\\);`,
            ),
          );
          expect(interop).not.toBeNull();
          expect(bundle).toContain(`${interop[3]} as cjsInterop`);
          expect(bundle).toContain(
            'export { default as defaultImport, namedImport } from "external-module";',
          );
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    },
  ],
};
