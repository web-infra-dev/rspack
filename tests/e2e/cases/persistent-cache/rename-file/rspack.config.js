const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  context: __dirname,
  // use production mod to make sure
  // the persistent cache will write to disk
  mode: 'production',
  plugins: [new rspack.HtmlRspackPlugin()],
  cache: {
    type: 'persistent',
  },
};
