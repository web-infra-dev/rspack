import path from 'node:path';
import fs from 'node:fs';
import rspack from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: async () => {
    const context = path.resolve(import.meta.dirname, 'src');
    const entries = { main: './src/main.js' };
    try {
      await fs.promises.stat(path.join(context, 'marker.js'));
      entries.shared = './src/shared.js';
    } catch {}
    return entries;
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
