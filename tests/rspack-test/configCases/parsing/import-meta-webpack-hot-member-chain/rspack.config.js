const { HotModuleReplacementPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  plugins: [new HotModuleReplacementPlugin()],
};
