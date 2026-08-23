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
        test: /dynamic\.js$/,
        type: 'javascript/dynamic',
      },
    ],
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  stats: {
    modules: true,
    nestedModules: true,
    optimizationBailout: true,
    providedExports: true,
  },
};
