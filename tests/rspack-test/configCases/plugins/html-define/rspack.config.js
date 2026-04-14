const HtmlRspackPlugin = require('html-rspack-plugin');
const { rspack } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    new HtmlRspackPlugin({
      template: './document.ejs',
    }),
    new rspack.DefinePlugin({
      title: JSON.stringify('CUSTOM TITLE'),
    }),
  ],
};
