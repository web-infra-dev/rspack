const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  entry: {
    polyfill: './polyfill.js',
    main: {
      dependOn: 'polyfill',
      import: './index.js',
    },
  },
  output: {
    filename: '[name].js',
  },
  target: 'web',
  optimization: {
    runtimeChunk: { name: 'runtime' },
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      __RSPACK_TEST_RUNTIME_MODE_RSPACK__: JSON.stringify(
        Boolean(globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK),
      ),
    }),
  ],
};
