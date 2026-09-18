/** @type {import('@rspack/core').Configuration} */
export default {
  entry: './index',
  stats: 'errors-warnings',
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
