const rspack = require('@rspack/core');

module.exports = {
  target: 'web',
  experiments: {
    fasterModuleConcatenation: true,
  },
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [new rspack.SubresourceIntegrityPlugin()],
};
