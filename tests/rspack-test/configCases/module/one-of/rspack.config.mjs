/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib.js/,
        use: ['./loader2.mjs'],
      },
      {
        test: /lib.js/,
        oneOf: [
          {
            resourceQuery: '/(__inline=false|url)/',
            use: ['./loader1.mjs'],
          },
          {
            use: ['./loader.mjs'],
          },
          {
            use: ['./loader1.mjs'],
          },
        ],
      },
    ],
  },
};
