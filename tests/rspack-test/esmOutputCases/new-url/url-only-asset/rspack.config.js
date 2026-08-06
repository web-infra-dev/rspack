module.exports = {
  experiments: {
    fasterModuleConcatenation: true,
  },
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
  },
  output: {
    assetModuleFilename: '[name][ext]',
  },
};
