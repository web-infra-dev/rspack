const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  entry: './index.js',
  cache: true,
  experiments: {
    cache: true,
  },
  plugins: [new rspack.HtmlRspackPlugin()],
};
