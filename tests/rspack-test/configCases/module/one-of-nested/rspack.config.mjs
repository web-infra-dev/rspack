/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /lib.js/,
        use: ['./loader2.js'],
      },
      {
        test: /lib.js/,
        oneOf: [
          {
            oneOf: [
              {
                resourceQuery: '/(__inline=false|url)/',
                use: ['./loader1.js'],
              },
              {
                use: ['./loader.js'],
              },
            ],
          },
          {
            use: ['./loader1.js'],
          },
        ],
      },
    ],
  },
};
