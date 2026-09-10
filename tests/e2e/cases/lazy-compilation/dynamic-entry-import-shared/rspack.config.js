import path from 'node:path';
import fs from 'node:fs';
import rspack from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: async () => {
    const context = path.resolve(import.meta.dirname, 'src');
    const files = await fs.promises.readdir(context);
    const result = {};
    for (const f of files) {
      if (f.endsWith('.js')) {
        result[path.basename(f, '.js')] = path.resolve(context, f);
      }
    }
    return result;
  },
  context: import.meta.dirname,
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
