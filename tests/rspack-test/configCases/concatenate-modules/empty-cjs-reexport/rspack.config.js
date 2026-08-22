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
    ],
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
};
