import path from 'node:path';
import fs from 'node:fs';
import rspack from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: async () => {
    const context = path.resolve(import.meta.dirname, 'src');
    const files = await fs.promises.readdir(context);
    let entries = files.filter((f) => f.startsWith('index'));
    entries.sort();
    return entries.reduce((acc, e, i) => {
      acc[`index${i + 1}`] = path.resolve(context, e);
      return acc;
    }, {});
  },
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new rspack.HtmlRspackPlugin()],
  devServer: {
    hot: true,
  },
  lazyCompilation: {
    entries: false,
  },
};
