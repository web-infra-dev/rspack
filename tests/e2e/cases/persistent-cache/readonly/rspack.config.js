const path = require('node:path');
const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: './index.js',
  context: __dirname,
  // use production mode to make sure the persistent cache will write to disk
  mode: 'production',
  plugins: [new rspack.HtmlRspackPlugin()],
  cache: {
    type: 'persistent',
    // Start with readonly: false so first build writes cache
    // Test will enable readonly for second build
    readonly: false,
    storage: { type: 'filesystem', directory: path.join(__dirname, '.cache') },
  },
};
