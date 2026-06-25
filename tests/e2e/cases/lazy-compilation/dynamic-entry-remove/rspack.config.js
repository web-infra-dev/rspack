const path = require('path');
const fs = require('fs');
const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: async () => {
    const context = path.resolve(__dirname, 'src');
    const files = await fs.promises.readdir(context);
    let entries = files.filter((f) => f.startsWith('index'));
    entries.sort();
    return entries.reduce((acc, e, i) => {
      acc[`index${i + 1}`] = path.resolve(context, e);
      return acc;
    }, {});
  },
  context: __dirname,
  mode: 'development',
  plugins: [
    new rspack.HtmlRspackPlugin({ chunks: ['index1'], filename: 'index.html' }),
    new rspack.HtmlRspackPlugin({
      chunks: ['index2'],
      filename: 'index2.html',
    }),
  ],
  devServer: {
    hot: true,
  },
  lazyCompilation: {
    entries: true,
  },
};
