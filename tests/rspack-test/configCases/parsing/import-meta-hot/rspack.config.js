const { HotModuleReplacementPlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  devtool: false,
  plugins: [new HotModuleReplacementPlugin()],
};
