/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./loader2.mjs', './loader1.mjs'],
        rules: [
          {
            test: /lib\.js$/,
            use: ['./loader.mjs'],
          },
        ],
      },
    ],
  },
};
