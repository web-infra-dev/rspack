const rspack = require('@rspack/core');

module.exports = {
  mode: 'development',
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new rspack.experiments.RslibPlugin()],
};
