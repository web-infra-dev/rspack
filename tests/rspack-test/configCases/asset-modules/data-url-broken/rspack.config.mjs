/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.(png|svg)$/,
        type: 'asset/inline',
      },
      {
        mimetype: 'image/svg+xml',
        type: 'asset/inline',
      },
      {
        test: /\.jpg$/,
        type: 'asset',
        parser: {
          dataUrlCondition: {
            maxSize: Infinity,
          },
        },
      },
      {
        mimetype: 'text/plain',
        type: 'asset/inline',
        loader: './loader',
      },
    ],
  },
};
