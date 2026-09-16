import { EntryPlugin } from '@rspack/core';
import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new EntryPlugin(
      import.meta.dirname,
      path.resolve(import.meta.dirname, './index.js'),
      {
        name: 'HtmlRspackPlugin_0-C:\\userCode\\x-project\\node_modules\\html-rspack-plugin\\lib\\loader.js!C:\\userCode\\x-project\\index.html',
        filename: 'index.js',
      },
    ),
  ],
};
