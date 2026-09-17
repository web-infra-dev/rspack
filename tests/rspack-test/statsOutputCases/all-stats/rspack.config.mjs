/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: 'eval',
  entry: './index.js',
  output: {
    filename: 'bundle.js',
  },
  module: {
    rules: [
      {
        mimetype: 'text/plain',
        type: 'asset',
      },
    ],
  },
  stats: { all: true },
};
