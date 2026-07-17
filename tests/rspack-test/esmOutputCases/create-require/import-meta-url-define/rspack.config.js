const { DefinePlugin } = require('@rspack/core');

module.exports = {
  module: {
    parser: {
      javascript: {
        createRequire: true,
        requireResolve: false,
      },
    },
  },
  plugins: [
    new DefinePlugin({
      'import.meta.url': JSON.stringify('file:///virtual/index.js'),
    }),
  ],
};
