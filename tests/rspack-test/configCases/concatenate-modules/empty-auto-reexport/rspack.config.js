const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  output: {
    library: {
      type: 'commonjs2',
    },
  },
  module: {
    rules: [
      {
        test: /empty\.js$/,
        sideEffects: true,
      },
      {
        test: /override-strict-empty\.js$/,
        parser: {
          overrideStrict: 'strict',
        },
      },
      {
        test: /override-non-strict-empty\.js$/,
        parser: {
          overrideStrict: 'non-strict',
        },
      },
      {
        test: /dynamic\.js$/,
        type: 'javascript/dynamic',
      },
    ],
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      EMPTY_AUTO_REEXPORT_DEFINED_EXPORTS: 'exports',
    }),
  ],
  stats: {
    modules: true,
    nestedModules: true,
    optimizationBailout: true,
    providedExports: true,
  },
};
