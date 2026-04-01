const { EntryPlugin } = require('@rspack/core');
const path = require('path');
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new EntryPlugin(__dirname, path.resolve(__dirname, './index.js'), {
      name: 'HtmlRspackPlugin_0-C:\\userCode\\x-project\\node_modules\\html-rspack-plugin\\lib\\loader.js!C:\\userCode\\x-project\\index.html',
      filename: 'index.js',
    }),
  ],
};
