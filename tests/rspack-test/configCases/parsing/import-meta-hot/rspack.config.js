const { HotModuleReplacementPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  devtool: false,
  externals: {
    fs: 'node-commonjs fs',
  },
  node: {
    __filename: false,
  },
  plugins: [new HotModuleReplacementPlugin()],
};
