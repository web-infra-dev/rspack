export default {
  mode: 'development',
  entry: './index.js',
  devtool: false,
  module: {
    rules: [
      {
        test: /a\.js/,
        with: {
          type: {
            not: 'raw',
          },
        },
        use: [
          {
            loader: './loader.mjs',
          },
        ],
      },
      {
        with: { type: 'raw' },
        type: 'asset/source',
      },
    ],
  },
};
