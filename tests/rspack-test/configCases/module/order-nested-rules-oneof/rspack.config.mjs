/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        use: ['./loader.mjs'],
        rules: [
          {
            test: /\.js$/,
            use: ['./loader1.mjs'],
          },
        ],
        oneOf: [
          {
            test: /lib\.js$/,
            use: ['./loader2.mjs'],
          },
          {
            test: /random-string/,
            use: ['./loader3.mjs'],
          },
        ],
      },
    ],
  },
};
