import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new rspack.EntryPlugin(import.meta.dirname, './a.js', {
      filename: () => 'pages/[name].js',
      name: 'a',
    }),
  ],
};
