const { DefinePlugin } = require('@rspack/core');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

module.exports = {
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
    rules: [
      {
        test: /index\.js$/,
        parser: {
          requireResolve: false,
        },
      },
    ],
  },
  plugins: [
    new DefinePlugin({
      'import.meta.url': JSON.stringify(
        pathToFileURL(path.join(__dirname, 'defined-context/index.js')).href,
      ),
    }),
  ],
};
