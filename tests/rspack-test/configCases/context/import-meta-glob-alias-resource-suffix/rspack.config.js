const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      'query-assets': path.join(__dirname, 'dir') + '?raw',
      'fragment-assets': path.join(__dirname, 'dir') + '#fragment',
      assets: path.join(__dirname, 'dir') + '?raw#fragment',
      'same-assets': path.join(__dirname, 'dir') + '?raw#fragment',
    },
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        include: [path.join(__dirname, 'dir'), path.join(__dirname, 'local')],
        use: path.join(__dirname, 'loader.cjs'),
      },
    ],
  },
};
