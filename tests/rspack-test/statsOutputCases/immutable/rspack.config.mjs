/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: 'eval',
  entry: './index.js',
  output: {
    filename: '[contenthash].js',
  },
  stats: {
    all: false,
    assets: true,
  },
};
