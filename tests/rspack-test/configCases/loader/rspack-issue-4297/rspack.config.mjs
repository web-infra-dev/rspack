/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib\.js$/,
        resourceQuery: /source/,
      },
      {
        test: /lib\.js$/,
        resourceQuery: { not: [/source/] },
        loader: './queryloader.mjs',
      },
      {
        test: /lib\.js$/,
        resourceFragment: /source/,
      },
      {
        test: /lib\.js$/,
        resourceFragment: { not: [/source/] },
        loader: './fragmentloader.mjs',
      },
    ],
  },
};
