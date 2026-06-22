const path = require('path');
const fs = require('fs');
const rspack = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: async () => {
    const context = path.resolve(__dirname, 'src');
    const files = await fs.promises.readdir(context);
    const result = {};
    for (const f of files) {
      if (f.endsWith('.js')) {
        result[path.basename(f, '.js')] = path.resolve(context, f);
      }
    }
    return result;
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
