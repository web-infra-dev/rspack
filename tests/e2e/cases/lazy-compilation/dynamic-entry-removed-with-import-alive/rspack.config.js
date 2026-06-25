const path = require('path');
const fs = require('fs');
const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: async () => {
    const context = path.resolve(__dirname, 'src');
    const entries = { main: './src/main.js' };
    try {
      await fs.promises.stat(path.join(context, 'marker.js'));
      entries.shared = './src/shared.js';
    } catch {}
    return entries;
  },
  context: __dirname,
  mode: 'development',
  plugins: [
    new rspack.HtmlRspackPlugin({ chunks: ['main'], filename: 'index.html' }),
    new rspack.HtmlRspackPlugin({
      chunks: ['shared'],
      filename: 'shared.html',
    }),
  ],
  devServer: {
    hot: true,
  },
  lazyCompilation: {
    entries: true,
    imports: true,
  },
};
