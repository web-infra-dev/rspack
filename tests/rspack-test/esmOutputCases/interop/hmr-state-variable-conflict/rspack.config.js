const { HotModuleReplacementPlugin } = require('@rspack/core');

module.exports = {
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new HotModuleReplacementPlugin()],
};
