const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      'query-assets': path.join(__dirname, 'dir') + '?raw',
      'fragment-assets': path.join(__dirname, 'dir') + '#fragment',
      assets: path.join(__dirname, 'dir') + '?raw#fragment',
      'same-assets': path.join(__dirname, 'dir') + '?raw#fragment',
      'raw-assets': path.join(__dirname, 'raw-dir') + '?raw#raw-fragment',
      'url-assets': path.join(__dirname, 'url-dir') + '?url#url-fragment',
    },
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        include: [
          path.join(__dirname, 'dir'),
          path.join(__dirname, 'local'),
          path.join(__dirname, 'raw-dir'),
          path.join(__dirname, 'url-dir'),
        ],
        use: path.join(__dirname, 'loader.cjs'),
      },
    ],
  },
};
