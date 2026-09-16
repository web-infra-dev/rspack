/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.js$/,
        use: ['./loader.js'],
        oneOf: [
          {
            test: /lib\.js$/,
            use: ['./loader1.js'],
          },
          {
            test: /random-string/,
            use: ['./loader2.js'],
          },
        ],
      },
    ],
  },
};
