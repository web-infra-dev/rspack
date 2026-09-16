/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        mimetype: 'image/gif',
        type: 'asset/resource',
        generator: {
          filename: '[name][ext][query]',
        },
      },
      {
        mimetype: 'text/html',
        type: 'asset/resource',
        generator: {
          filename: '[name].[contenthash][ext]',
        },
      },
      {
        mimetype: 'image/png',
        type: 'asset/resource',
        generator: {
          filename: '[contenthash][ext][query]',
        },
      },
      {
        mimetype: 'image/svg',
        type: 'asset/resource',
      },
    ],
  },
};
