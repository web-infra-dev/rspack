const { rspack } = require('@rspack/core');

/** @type { import('@rspack/core').RspackOptions } */

module.exports = {
  context: __dirname,
  entry: './src/index.js',
  stats: 'none',
  plugins: [new rspack.HtmlRspackPlugin({ template: './src/index.html' })],
};
