const rspack = require('@rspack/core');

module.exports = {
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [new rspack.SubresourceIntegrityPlugin()],
};
