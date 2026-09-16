/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./loader2.js', './loader1.js'],
        rules: [
          {
            test: /lib\.js$/,
            use: ['./loader.js'],
          },
        ],
      },
    ],
  },
};
